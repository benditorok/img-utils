# img-utils

Image processing in Rust using [libcudaimg](https://github.com/benditorok/libcudaimg/tree/main).

## Build process

This project currently runs only on Windows. For the _libcudaimg_ library to build you need to have some **CUDA Runtime** installed as well as **Visual studio 22**. In the future I plan to convert the _libcudaimg_ library to a CMake project.

- Clone using the **--recursive** switch to get the _libcudaimg_ submodule
- Run `libcudaimg_build.bat` to compile the _libcudaimg_ library
  - _Note: you might need to adjust the location of the CUDA and/or Visual Studio installation_
- `cargo run`

## Current features

- Invert images
- Gamma transformation
- Logarithmic transformation
- Grayscale conversion
- Histogram plotting
