# Adding dependencies

To make adding dependencies easier, we'll install a cargo extension called [cargo-edit](https://github.com/killercup/cargo-edit).

```bash
cargo install cargo-edit
```

Then we can use it to add some dependencies to our `phantom_dependencies` crate.

```bash
cargo add anyhow -p phantom_dependencies
cargo add env_logger -p phantom_dependencies 
cargo add log -p phantom_dependencies
cargo add thiserror -p phantom_dependencies
```

Now we re-export these dependencies in `phantom_dependencies/lib.rs`:

```rust,noplaypen
pub use anyhow;
pub use env_logger;
pub use log;
pub use thiserror;
```

We will continue following this pattern whenever we add dependencies to the project!

The dependencies we have added are:

- [anyhow](https://github.com/dtolnay/anyhow) 
  - A flexible concrete Error type built on `std::error::Error`. We will use this in our applications, which do not need to return detailed, complex error types.
- [thiserror](https://github.com/dtolnay/thiserror)
  - A `derive(Error)` for struct and enum error types. We will use this in our engine and its libraries, so that we can return detailed, descriptive error types. This will ultimately make debugging easier.
- [log](https://github.com/rust-lang/log)
  - The standard Rust logger facade.
- [env_logger](https://github.com/dtolnay/anyhow)
  - An implementation of the standard rust logger facade that is configured using environment variables.
