# cuda-imgproc

Image processing in Rust using a CUDA library.

## Build process

- Clone using the **--recursive** switch to get the _libcudaimg_ submodule
- Run `libcudaimg_build.bat` to compile the _libcudaimg_ library
  - _Note: you might need to adjust the location of the CUDA and/or Visual Studio installation_
- `cargo run`

## Current features

- Invert images
