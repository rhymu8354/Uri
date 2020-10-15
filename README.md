# Uri (rhymuri)

This is a library which implements [IETF RFC
3986](https://tools.ietf.org/html/rfc3986), "Uniform Resource Identifier (URI):
Generic Syntax".

[![Crates.io](https://img.shields.io/crates/v/rhymuri.svg)](https://crates.io/crates/rhymuri)
[![Documentation](https://docs.rs/rhymuri/badge.svg)][dox]

More information about the Rust implementation of this library can be found in
the [crate documentation][dox].

[dox]: https://docs.rs/rhymuri

A URI is a compact sequence of characters that identifies an abstract or
physical resource.  One common form of URI is the Uniform Resource Locator
(URL), used to reference web resources:

    http://www.example.com/foo?bar#baz

Another kind of URI is the path reference:

    /usr/bin/zip

The purpose of this library is to provide a `Uri` type to represent a URI,
with functions to parse URIs from their string representations, as well as
assemble URIs from their various components.

This is a multi-language library containing independent implementations
for the following programming languages:

* C++
* Rust

## Building the C++ Implementation

A portable library is built which depends only on the C++11 compiler and
standard library, so it should be supported on almost any platform.  The
following are recommended toolchains for popular platforms.

* Windows -- [Visual Studio](https://www.visualstudio.com/) (Microsoft Visual
  C++)
* Linux -- clang or gcc
* MacOS -- Xcode (clang)

This library is not intended to stand alone.  It is intended to be included in
a larger solution which uses [CMake](https://cmake.org/) to generate the build
system and build applications which will link with the library.

There are two distinct steps in the build process:

1. Generation of the build system, using CMake
2. Compiling, linking, etc., using CMake-compatible toolchain

### Prerequisites

* [CMake](https://cmake.org/) version 3.8 or newer
* C++11 toolchain compatible with CMake for your development platform (e.g.
  [Visual Studio](https://www.visualstudio.com/) on Windows)

### Build system generation

Generate the build system using [CMake](https://cmake.org/) from the solution
root.  For example:

```bash
mkdir build
cd build
cmake -G "Visual Studio 15 2017" -A "x64" ..
```

### Compiling, linking, et cetera

Either use [CMake](https://cmake.org/) or your toolchain's IDE to build.
For [CMake](https://cmake.org/):

```bash
cd build
cmake --build . --config Release
```

## License

Licensed under the [MIT license](LICENSE.txt).
