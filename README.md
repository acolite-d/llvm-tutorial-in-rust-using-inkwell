# A Rust Rewrite of the LLVM Tutorial, using Inkwell
Original tutorial found here https://llvm.org/docs/tutorial/#kaleidoscope-implementing-a-language-with-llvm. Have rewritten everything up to Part 4. Code is more or less the same, but uses less global state, more modularity, more organization.

The code is material for these blog posts:
- Post 1
- Post 2

## Building

In order to build you will need the following:

- Rust Compiler and associated toolchain, please use https://rustup.rs/
- LLVM, either built from source or installed via package manager. Code has been tested with version 17.0.6, but inkwell can support anywhere from version 4-18 at the moment.

Be sure the installation of LLVM is locatable within your PATH, run `cargo run` to compile and run interactive REPL session.