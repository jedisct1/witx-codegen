![WITX code generator](logo.png)
================================

# WITX-CodeGen: A WITX code generator

WITX is a way to describe types and function interfaces from WebAssembly modules.

From this, code generators can generate code that accesses data and calls functions from different languages using the same layout and calling conventions.

WITX is the description language used by [WASI](https://wasi.dev). This tool uses the next (as on April 5th, 2021) revision of the format, as implemented in version 0.10 of the Rust `witx` crate.

WITX-CodeGen is written in Rust, but was designed to generate code for multiple languages that is simple to use, as well as multiple documentation formats.

Backends:

* [X] AssemblyScript
* [ ] Zig
* [ ] Rust
* [ ] TinyGo
* [ ] C/C++
* [X] API Overview
* [ ] Documentation

`witx-codegen` is a rewrite of `as-witx`.
