# Project Setup

> If you wish to make an apple pie from scratch,
>
> you must first invent the universe.
>
> ~ Carl Sagan

Welcome to the exciting world of game development!

To get started, we'll first need to setup our project structure.

## Creating the Project Structure

Let's create a new project! We will call our engine the `phantom` game engine.

```bash
cargo new --lib phantom
cd phantom
```

Now we can create all of the libraries and applications we will need for this project.

```bash
# Applications
cargo new --vcs none apps/editor
cargo new --vcs none apps/viewer

# Libraries
cargo new --lib crates/phantom_app
cargo new --lib crates/phantom_dependencies
cargo new --lib crates/phantom_gui
cargo new --lib crates/phantom_render
cargo new --lib crates/phantom_world
```

### A Window Icon

For our window, we'll want a nice looking icon. Let's copy the following png into a folder for later use.

![phantom-icon](images/phantom.png)

```bash
mkdir assets/icons
pushd assets/icons
curl -O https://matthewjberger.xyz/phantom/images/phantom.png
popd
```

### Code Linting

To perform code linting we'll use [clippy](https://github.com/rust-lang/rust-clippy).

First, let's install `clippy`:

```bash
rustup update
rustup component add clippy
```

And then add a configuration file at the root named `clippy.toml` with the following contents:

```toml
too-many-lines-threshold = 80
too-many-arguments-threshold = 5
```

> These can be any settings you like, of course. [Here is a list of valid clippy options](https://rust-lang.github.io/rust-clippy/master/).

Lints will be performed automatically by `vscode`. To lint manually, you can run `cargo clippy -p phantom`.

### Code Formatting

To perform code formatting we'll use [rustfmt](https://github.com/rust-lang/rustfmt).

First, let's install `rustfmt`:

```bash
rustup update
rustup component add rustfmt
```

And then add a configuration file at the root named `rustfmt.toml` with the following contents:

```toml
max_width = 100
```

> These can be any settings you like, of course. [Here is a list of valid rustfmt settings](https://rust-lang.github.io/rustfmt/?version=v1.5.0&search=).

Formatting can be performed automatically by `vscode`. To format the project manually, you can run `cargo fmt --workspace`.

### Add a Readme

Our `README.md` looks like this:

```markdown
# Phantom

Phantom is a 3D game engine written in Rust!

## Development Prerequisites

- [Rust](https://www.rust-lang.org/)

## Instructions

To run the visual editor for Phantom, run this command in the root directory:

`cargo run --release --bin editor`.

```

## Putting It All Together

Now to connect our existing projects to one another, we'll have to update the contents of some of our new source files.

### Connecting Our Libraries

Our game engine is designed as a library. We will want to make the various parts of the engine accessible by re-exporting them.

Our `Cargo.toml` at the root should look like this:

```toml
[package]
name = "phantom"
version = "0.1.0"
edition = "2021"

[workspace]
default-members = ["apps/*"]
members = ["apps/*", "crates/*"]

[dependencies]
phantom_app = { path = "crates/phantom_app" }
phantom_dependencies = { path = "crates/phantom_dependencies" }
phantom_gui = { path = "crates/phantom_gui" }
phantom_render = { path = "crates/phantom_render" }
phantom_world = { path = "crates/phantom_world" }
```

Next, the `src/lib.rs` should look like this:

```rust,noplaypen
pub mod app {
    pub use phantom_app::*;
}

pub mod dependencies {
    pub use phantom_dependencies::*;
}

pub mod gui {
    pub use phantom_gui::*;
}

pub mod render {
    pub use phantom_render::*;
}

pub mod world {
    pub use phantom_world::*;
}
```

This lets us access the public exports of all of our engine libraries, and applications will only need
a single import of our main engine library.

### Handling Dependencies

To handle dependencies consistently across all of our projects,
we have created a `phantom_dependencies` project. Here will we
list all of our dependencies and re-export them. This helps
ensure the same version of any given dependency is used across all of our modules.

For the following apps:

* `phantom_app`
* `phantom_gui`
* `phantom_render`
* `phantom_world`

Add our `phantom_dependencies` crate as a dependency in the corresponding `Cargo.toml`:

```toml
phantom_dependencies = { path = "../phantom_dependencies" }
```

> Some macros only work if the crate they are exported from is included in the dependencies for the project it is used in, but this will cover the majority of our dependencies.

### Connecting our Apps

For the following apps:

* `editor`
* `viewer`

Add our main `phantom` crate as a dependency in the corresponding `Cargo.toml`:

```toml
phantom = { path = "../.." }
```

## Verifying your Project

Check your project so far with the following command:

```bash
cargo check
```
