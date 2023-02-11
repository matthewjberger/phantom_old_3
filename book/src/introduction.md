# Introduction

This book will walk you through the creation and design of a basic 3D game using the [Phantom game engine](https://github.com/matthewjberger/phantom).

## Why Rust?

Rust is a great alternative to `C/C++`!  A few of the benefits:

* It provides a smooth workflow for developers with clear, specific error messages from the compiler
* [rustup](https://rustup.rs/) and [cargo](https://github.com/rust-lang/cargo) make managing rust toolchain installations and rust projects straightforward
* The lints from [clippy](https://github.com/rust-lang/rust-clippy) help to improve the code quality and catch certain common mistakes
* [rustfmt](https://github.com/rust-lang/rustfmt) handles formatting the code and code style
* Memory safety. Code written outside of `unsafe` blocks is checked by the [borrow checker](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html?highlight=borrow#references-and-borrowing)
