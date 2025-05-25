use std::ffi::c_void;
use std::ops::Deref;

use raw::{
    cudaError_cudaSuccess, cudaGraphicsD3D11RegisterResource, cudaGraphicsResource,
    cudaGraphicsUnregisterResource,
};
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11::{self as wind3d, ID3D11Buffer, ID3D11DeviceContext};

mod raw {
    #![allow(
        non_snake_case,
        non_camel_case_types,
        non_upper_case_globals,
        dead_code
    )]
    include!(concat!(env!("OUT_DIR"), "/cuda_interop.rs"));
}

pub type CudaResourceHandle = *mut cudaGraphicsResource;
pub struct CudaD3D11Resource {
    handle: *mut cudaGraphicsResource,
    pub resource: ID3D11Buffer,
    pub device_context: ID3D11DeviceContext,
}

unsafe impl Send for CudaD3D11Resource {}

impl CudaD3D11Resource {
    /// Register a D3D11 resource (i.e Buffer) with CUDA.
    pub fn new(
        device_context: ID3D11DeviceContext,
        resource: ID3D11Buffer,
    ) -> windows::core::Result<Self> {
        let mut handle: *mut cudaGraphicsResource = std::ptr::null_mut();
        let raw_ptr = resource.as_raw() as *mut _;
        let reg_result = unsafe { cudaGraphicsD3D11RegisterResource(&mut handle, raw_ptr, 0) };
        if reg_result != cudaError_cudaSuccess {
            return Err(windows::core::Error::from_win32());
        }
        Ok(Self {
            handle,
            resource,
            device_context,
        })
    }

    /// If you just want the graphics resource back as a raw pointer.
    pub unsafe fn raw(&self) -> *mut cudaGraphicsResource {
        self.handle
    }
    
    pub fn resource(&self) -> &ID3D11Buffer {
        &self.resource
    }
    
    pub fn device_context(&self) -> &ID3D11DeviceContext {
        &self.device_context
    }

    fn map_resource(&mut self, stream_ptr: *mut c_void) -> anyhow::Result<CudaMappedResource> {
        let result = CudaMappedResource::new(self, stream_ptr);
        if let Err(e) = result {
            Err(anyhow::format_err!("Failed to map resource, error: {}", e))
        } else {
            Ok(result.unwrap())
        }
    }

    pub fn copy_from(&mut self, buf: ID3D11Buffer) {
        // SAFETY: we hold an exclusive reference to self here, so the resource cannot be mapped while calling this
        unsafe {
            self.device_context
                .CopyResource(&self.resource.clone(), &buf);
        }
    }

    /// Anything you want to do with the device pointer, do it in here.
    /// Just don't try to do anything with the registered memory,
    /// like dispatch a compute shader on the mapped memory
    pub fn with_mapped<R, E, F>(&mut self, stream_ptr: *mut c_void, f: F) -> Result<R, E>
    where
        F: FnOnce(CudaMappedResource) -> Result<R, E>,
        E: From<anyhow::Error>,
    {
        let mapped_resource = self
            .map_resource(stream_ptr)
            .map_err(|e| anyhow::anyhow!(e))?;
        f(mapped_resource)
    }
}

impl Drop for CudaD3D11Resource {
    fn drop(&mut self) {
        // SAFETY: we registered this handle in `new`, so it’s ours to unregister
        let err = unsafe { cudaGraphicsUnregisterResource(self.handle) };
        if err != cudaError_cudaSuccess {
            eprintln!("Failed to unregister CUDA resource: {:?}", err);
        }
    }
}

/// A guard that maps a CUDA‐registered D3D11 resource on creation
/// and automatically unmaps it on drop.
pub struct CudaMappedResource<'a> {
    // take an exclusive reference to the underlying resource, so that we can't map or unmap twice
    cuda_res: &'a mut CudaD3D11Resource,
    stream_ptr: *mut std::ffi::c_void,
    dev_ptr: *mut std::ffi::c_void,
    size: usize,
}

impl<'a> CudaMappedResource<'a> {
    /// Maps the resource into CUDA address space and returns
    /// the device pointer and size.
    pub fn new(
        cuda_res: &'a mut CudaD3D11Resource,
        stream_ptr: *mut std::ffi::c_void,
    ) -> Result<Self, raw::cudaError_t> {
        // SAFETY: map the resource
        let err =
            unsafe { raw::cudaGraphicsMapResources(1, &mut cuda_res.handle, stream_ptr as _) };
        if err != cudaError_cudaSuccess {
            return Err(err);
        }

        let mut dev_ptr = std::ptr::null_mut();
        let mut size = 0;
        let err = unsafe {
            raw::cudaGraphicsResourceGetMappedPointer(&mut dev_ptr, &mut size, cuda_res.handle)
        };
        if err != cudaError_cudaSuccess {
            // unmap on error
            unsafe { raw::cudaGraphicsUnmapResources(1, &mut cuda_res.handle, stream_ptr as _) };
            return Err(err);
        }

        Ok(Self {
            cuda_res,
            stream_ptr,
            dev_ptr,
            size: size as usize,
        })
    }

    /// Get the raw device pointer and byte‐size.
    pub fn as_ptr(&'a self) -> (*mut c_void, usize) {
        (self.dev_ptr, self.size)
    }
}

impl<'a> Drop for CudaMappedResource<'a> {
    fn drop(&mut self) {
        // SAFETY: automatically unmap when the guard goes out of scope
        let err = unsafe {
            raw::cudaGraphicsUnmapResources(1, &mut self.cuda_res.handle, self.stream_ptr as _)
        };
        if err != cudaError_cudaSuccess {
            eprintln!("Failed to unmap CUDA resource: {:?}", err);
        }
    }
}