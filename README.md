# Cuda D3D11 Interop

For when you need to register and unregister Direct3D11 buffers with CUDA, as well as map/unmap those buffers for CUDA kernels.

# Prerequisites

CUDA Toolkit is installed.

Environment variable `CUDA_PATH` points to your CUDA install.

For example:

- On windows, something like: `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\vXX.X` where `XX.X` is the version you have installed
- On linux: `/usr/local/cuda`


# Quickstart

```rust
/// When we have device_ctx: ID3D11DeviceContext and d3d_buffer: ID3D11Buffer
/// both from the Windows API through a crate like `windows` or `winapi`
let mut cuda_res = CudaD3D11Resource::new(device_ctx, d3d_buffer)?;
cuda_res.with_mapped(stream, |mapped| {
    let (dev_ptr, size) = mapped.as_ptr();
    // use dev_ptr in CUDA kernel launch
    Ok(())
})?;
```

# License

MIT

