use std::ops::Deref;

use raw::{cudaError_cudaSuccess, cudaGraphicsD3D11RegisterResource};
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D11 as wind3d;

mod raw {
    #![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/cuda_interop.rs"));
}

pub fn register_d3d11_texture2d_with_cuda(
    tex: &wind3d::ID3D11Texture2D,
) -> Result<raw::cudaGraphicsResource_t, raw::cudaError_t> {
    let resource: &windows::Win32::Graphics::Direct3D11::ID3D11Resource = &tex.deref();
    let raw_ptr = resource.as_raw() as *mut _;

    let mut handle = std::ptr::null_mut();
    let reg_result = unsafe { cudaGraphicsD3D11RegisterResource(&mut handle, raw_ptr, 0) };

    if reg_result == cudaError_cudaSuccess {
        Ok(handle)
    } else {
        Err(reg_result)
    }
}
