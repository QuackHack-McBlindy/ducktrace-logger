# **ducktrace-logger**

  
`ducktrace-logger` is a duck themed application logging system written in Rust that is configured using environment variables.  
It only prints (and saves to `~/.config/duckTrace/`) logs at or above `$DT_LOG_LEVEL`.  
It also provides a handy `dt_timer` for measuring operations.  
  

## Installation

  
Add as a git dependency in your `Cargo.toml`:  

```toml
[dependencies]
ducktrace-logger = { git = "https://github.com/QuackHack-McBlindy/ducktrace-logger" }
```

## Configuration


The logger can be configured via environment variables (preferred) or programmatically.  
  

### Environment variables (easiest)  

- `DT_LOG_LEVEL` – set to `DEBUG`, `INFO`, `WARNING`, `ERROR`, or `CRITICAL` (default: `INFO`)
- `DEBUG=1` – enable debug messages even if level is higher
- `DT_LOG_PATH` – directory for log files (default: `~/.config/duckTrace/`)
- `DT_LOG_FILE` – filename inside that directory (default: `unknown-script.log`)
  
  
### Programmatic setup (optional)

  
If you need to set the log level from code (ignoring env vars), call `setup_ducktrace_logging` **before any logging**:  
  

```rust
use ducktrace_logger::*;

fn main() {
    // Must come first!
    dt_setup(None, Some("DEBUG"));
    
    dt_info("This will now show debug messages if DEBUG mode is on");
}
```


## **Basic usage:**

```rust
use ducktrace_logger::*;

fn main() {
    dt_info("Application started");
    dt_debug("Some debugging message");

    let timer = dt_timer("my operation");
    // ... do work ...
    timer.lap("first phase");
    // ... more work ...
    timer.complete();
}
```


## Error output  

  
  
```rust
use ducktrace_logger::*;

fn main() {
    dt_error("The database connection failed");
}
```

  
This will print to stderr:  
  
```bash
[🦆📜] [14:35:22] ❌ERROR❌ ⮞ The database connection failed

🦆 duck say ⮞ fuck ❌ The database connection failed
```

  
The duck line is only shown for Error and Critical levels – it does not appear for Debug, Info, or Warning. It is also not written to the log file, only to the console.  
