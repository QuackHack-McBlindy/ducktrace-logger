# **ducktrace-logger**


`ducktrace-logger` is a duck themed application logging system written in Rust that is configured using environment variables.  
It only prints (and saves to `~/.config/duckTrace/`) logs at or above `$DT_LOG_LEVEL`.  
It also provides a handy `dt_timer` for measuring operations.  


- **🦆 Works seamlessly with:**
  - [ducktrace-tui](https://github.com/quackhack-mcblindy/ducktrace-tui) *(TUI for browsing logs and managing services)* 


<br>   

## **Installation**

  
Add **ducktrace-logger** as a dependency in `Cargo.toml`.

```toml
[dependencies]
ducktrace-logger = "0.1.4"
```
  

## **Configuration**


The logger can be configured via environment variables (preferred) or programmatically.  
  

### Environment variables (easiest)  

- `DT_LOG_LEVEL` – set to `DEBUG`, `INFO`, `WARNING`, `ERROR`, or `CRITICAL` (default: `INFO`)
- `DT_LOG_PATH` – directory for log files (default: `~/.config/duckTrace/`)
- `DT_LOG_FILE` – filename inside that directory (default: `unknown-script.log`)
  
  
### Programmatic setup (optional)

  
If you need to set the log level from code (ignoring env vars), call `setup_ducktrace_logging` **before any logging**:  
  

```rust
use ducktrace_logger::*;

fn main() {
    // Option 1: Use environment variables (DT_LOG_PATH, DT_LOG_FILE, DT_LOG_LEVEL)
    //dt_setup(None, None);
    // Above is same as:
    dt_info("Skipping setup uses env vars");

    // Option 2: Override only the log file (uses default directory if no path given)
    dt_setup(Some("myapp.log"), None);

    // Option 3: Override only the log level
    dt_setup(None, Some("DEBUG"));

    // Option 4: Override both file and level
    dt_setup(Some("/var/log/myapp.log"), Some("ERROR"));

    dt_debug("This is a quackin' debuggin' message!");
    // Arguments can be passed:
    dt_debug!("Example: {} <-- there", example_id);
    // That works for all log levels
    dt_info!("Example: {} <-- there", example_id);    
    // ...    
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
