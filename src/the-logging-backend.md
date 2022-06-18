# The Logging Backend

In order for the logger facade functions to work, we will need to add a logging backend. For this project, we'll use [simplelog](https://github.com/drakulix/simplelog.rs).

Add this dependency to `crates/obsidian_app/Cargo.toml`:

```toml
simplelog = { version = "0.9.0", features = ["termcolor"] }
```

> Note: The `termcolor` feature allows for colored terminal log output.

Now we can create a logger module to setup the logger backend.

```rust,noplaypen
// crates/obsidian_app/src/lib.rs
mod logger;
...
```

```rust,noplaypen
// crates/obsidian_app/src/logger.rs
use anyhow::{Context, Result};
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::{fs::File, path::Path};

pub fn create_logger(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display().to_string();
    let error_message = format!("Failed to create log file named: {}", name);
    let file = File::create(path).context(error_message)?;
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(LevelFilter::max(), Config::default(), file),
    ])?;
    Ok(())
}
```

The [CombinedLogger](https://docs.rs/simplelog/0.9.0/simplelog/struct.CombinedLogger.html) lets us create a [TermLogger](https://docs.rs/simplelog/0.9.0/simplelog/struct.TermLogger.html) and a [WriteLogger](https://docs.rs/simplelog/0.9.0/simplelog/struct.WriteLogger.html) at the same time. We will only log messages with the severity [Info](https://docs.rs/simplelog/0.9.0/simplelog/enum.Level.html#variant.Info) and above to the terminal, and we will log all messages to the configuration file.

And finally we can invoke the `create_logger` function in our `run_application` method:

```rust,noplaypen
pub fn run_application(mut runner: impl Run + 'static, configuration: AppConfig) -> Result<()> {
    create_logger(&configuration.logfile_name)?;
    ...
}
```

