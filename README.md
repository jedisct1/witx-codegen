![WITX code generator](logo.png)
================================

![CI status](https://github.com/jedisct1/witx-codegen/actions/workflows/ci.yml/badge.svg)
[![crates.io](https://img.shields.io/crates/v/witx-codegen.svg)](https://crates.io/crates/witx-codegen)

# WITX-CodeGen: A WITX code and documentation generator

WITX is a way to describe types and function interfaces for WebAssembly modules.

From this, code generators can produce code to access data, call or implement functions from different languages using the same layout and calling conventions.

WITX-CodeGen doesn't do transformations when functions are called. Instead, it exposes types that have the same layout in all languages, like a zero-copy serialization format. Data can thus be easily shared between guests and hosts without any overhead.

The generated code is compatible with the WebAssembly standard APIs ([WASI](https://wasi.dev)).

This tool uses the next (as on June 9th, 2021) revision of the format definition, that will eventually be required for interface types.

`witx-codegen` is currently written in Rust, but it is totally language-agnostic. It is also compatible with all WebAssembly runtimes. The generated code is optimized for simplicity and readability.

The tool can also produce different documentation formats.

`witx-codegen` supersedes `as-witx`, `zig-witx`, `witx-docgen`, `witx-overview-docgen` and `witx-generate-raw`.

## Installation

* Via `cargo`:

```sh
cargo install witx-codegen
```

* Precompiled binaries: tarballs and Debian/Ubuntu packages are available [here](https://github.com/jedisct1/witx-codegen/releases/latest).

## Usage

```text
WITX code generator for WebAssembly guest modules

USAGE:
    witx-codegen [FLAGS] [OPTIONS] <witx_files>...

FLAGS:
    -h, --help            Prints help information
    -H, --skip-header     Do not generate a header
    -I, --skip-imports    Ignores imported types and functions
    -V, --version         Prints version information

OPTIONS:
    -m, --module-name <module_name>
            Set the module name to use instead of reading it from the witx file

    -o, --output <output_file>         Output file, or - for the standard output
    -t, --output-type <output_type>
            Output type. One in: {assemblyscript, zig, rust, overview, markdown}
            [default: assemblyscript]

ARGS:
    <witx_files>...    WITX files
```

## Backends

* [X] Markdown documentation ([example](https://github.com/jedisct1/witx-codegen/blob/master/example-output/markdown.md))
* [X] API Overview ([example](https://github.com/jedisct1/witx-codegen/blob/master/example-output/overview.txt))
* [X] AssemblyScript ([example](https://github.com/jedisct1/witx-codegen/blob/master/example-output/assemblyscript.ts))
* [X] Zig ([example](https://github.com/jedisct1/witx-codegen/blob/master/example-output/zig.zig))
* [X] Rust ([example](https://github.com/jedisct1/witx-codegen/blob/master/example-output/rust.rs))
* [ ] TinyGo
* [ ] C/C++
* [ ] Swift
* [ ] HTML documentation

Support for additional languages is more than welcome!

## Example inputs

See the [`test`](https://github.com/jedisct1/witx-codegen/tree/master/test) folder for examples of WITX input files.

Other input formats may also be eventually supported, as well as extensions to produce more structured documentation.

## WITX format

See the `test` directory for some examples.

### Basic types

`bool`, `char`, `u8`, `u16`, `u32`, `u64`, `s8`, `s16`, `s32`, `s64`

### Other types

* `string`: a read-only string.
* `(in-buffer u8)`: a read-only buffer whose elements are of type `u8`.
* `(out-buffer u8)`: a buffer whose elements are of type `u8`.
* `(@witx const_pointer u8)`: a read-only `u8` pointer.
* `(@witx pointer u8)`: a `u8` pointer.
* `(@witx usize)`: an object size.

### Type aliases

* `(typename $status_code u16)`
* `(typename $size (@witx usize))`

Note that returned values from function must all be aliases, not raw types.

### Handles

Handles are opaque references to objects managed by the host.

In order to use handles, a "resource" has to be declared:

```
(resource $http_handle)
```

A "resource" represent a group of handles. The same resource can be shared by all handle types from the same module.

Each handle type can then be declared as aliases:

```
(typename $query (handle $http_handle))
(typename $response_handle (handle $http_handle))
```

### Constants

```
(typename $big_int u64)
(@witx const $big_int $zero 0)
(@witx const $big_int $a_hundred 100)
(@witx const $big_int $a_big_value 0xff00000000000000)
(@witx const $big_int $a_bigger_value 0xffffffffffffffff)
```

### Structures

```
(typename $example_structure
  (record
    (field $first_member bool)
    (field $second_member u8)
    (field $third_member string)
  )
)
```

Structures that only contain booleans are encoded as bit sets.

### Tuples

```
(typename $test_tuple (tuple $test_bool $test_medium_int $big_int))
```

### Tagged unions

```
(typename $test_tagged_union
  (variant (@witx tag u16)
    (case $first_choice u8)
    (case $second_choice string)
    (case $third_choice f32)
    (case $empty_choice)
  )
)
```

This defines a union with a tag representing the active member. The example above generates a structure equivalent to:

```zig
struct {
    tag: u16,
    member: union {
        first_choice: u8,
        second_choice: string,
        third_choice: f32,
        empty_choice: (),
    }
}
```

### Imports

Import some aliases, or all of them, from `common.witx`:

```
(use $some_type, $some_other_type from $common)
(use * from $common)
```

### Modules

Only one module can be present in a file, whose name must match the module name. A module is defined as follows:

```
(module $module_name
...
)
```

It contains everything: types, handles, functions and imports.

### Functions

```
(@interface func (export "symmetric_key_generate")
 (param $algorithm string)
 (param $options $opt_options)
 (result $error (expected $symmetric_key (error $crypto_errno)))
)
```

This declares a `symmetric_key_generate` function, with two input parameters (`algorithm` and `options` of type `string` and `opt_options`).

The function returns an error code of type `$crypto_errno`. Or, if no error occurred, the function returns a value of type `$symmetric_key`.

In Rust, an equivalent function would be:

```rust
fn symmetric_key_generate(algorithm: &str, options: OptOptions)
  -> Result<SymmetricKey, CryptoErrno>;
```

Returning multiple values:

```
(@interface func (export "symmetric_key_id")
  (param $key $symmetric_key)
  (param $key_id (@witx pointer u8))
  (param $key_id_max_len $size)
  (result $error (expected (tuple $size $version) (error $crypto_errno)))
)
```

The function returns either an error, or two values, of type `$size` and `$version`.

The above example is eauivalent to a declaration like that one in Rust:

```rust
fn symmetric_key_id(key: SymmetricKey, key_id: *mut u8, key_id_max_len: usize)
  -> Result<(Size, Version), CryptoErrno>;
```
