pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

use std::{
    env,
    fs::{OpenOptions, File},
    io::Write,
    sync::{OnceLock, Mutex},
    time::Instant,
};
use chrono::Local;
use colored::*;

static LOGGER: OnceLock<Mutex<DuckTraceLogger>> = OnceLock::new();


struct DuckTraceLogger {
    level: LogLevel,
    log_file: Option<File>,
    debug_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl DuckTraceLogger {
    fn log_config(&mut self) {
        let log_path_display = match &self.log_file {
            Some(_) => {
                let log_path = env::var("DT_LOG_PATH").unwrap_or_else(|_| "~/.config/duckTrace".to_string());
                let log_file = env::var("DT_LOG_FILE").unwrap_or_else(|_| "unknown-script.log".to_string());
                format!("{}/{}", log_path, log_file)
            }
            None => "Logging to file disabled".to_string(),
        };
        let level_str = match self.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        };
        self.log(LogLevel::Debug, &format!("Logger initialized: level={}, file={}, debug_mode={}", 
            level_str, log_path_display, self.debug_mode));
    }

    fn new(level_str: Option<&str>) -> Self {
        let debug_mode = env::var("DEBUG").is_ok();
        let level = match level_str {
            Some(l) => Self::level_from_str(l),
            None => match env::var("DT_LOG_LEVEL")
                .unwrap_or_else(|_| "INFO".to_string())
                .to_uppercase()
                .as_str()
            {
                "DEBUG" => LogLevel::Debug,
                "WARNING" => LogLevel::Warning,
                "ERROR" => LogLevel::Error,
                "CRITICAL" => LogLevel::Critical,
                _ => LogLevel::Info,
            },
        };
        
        let log_file = Self::setup_log_file();
        let mut logger = Self { level, log_file, debug_mode };
        logger.log_config();
        logger
    }
    
    fn level_from_str(s: &str) -> LogLevel {
        match s.to_uppercase().as_str() {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            "CRITICAL" => LogLevel::Critical,
            _ => LogLevel::Info,
        }
    }
    

    fn setup_log_file() -> Option<File> {
        let log_path = env::var("DT_LOG_PATH")
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config/duckTrace", home)
            });

        std::fs::create_dir_all(&log_path).ok()?;

        let log_filename = env::var("DT_LOG_FILE")
            .unwrap_or_else(|_| "unknown-script.log".to_string());

        let full_path = std::path::Path::new(&log_path).join(log_filename);

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)
            .ok()
    }
    
    fn should_log(&self, msg_level: LogLevel) -> bool {
        if msg_level == LogLevel::Debug && !self.debug_mode {
            return false;
        }
        msg_level >= self.level
    }
    
    fn get_symbol(&self, level: LogLevel) -> &'static str {
        match level {
            LogLevel::Debug => "⁉️",
            LogLevel::Info => "✅",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Critical => "🚨",
        }
    }
    
    fn format_message(&self, level: LogLevel, message: &str) -> String {
        let timestamp = Local::now().format("%H:%M:%S");
        let symbol = self.get_symbol(level);
        let level_str = match level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        };
        
        format!("[🦆📜] [{}] {}{}{} ⮞ {}", 
            timestamp, symbol, level_str, symbol, message)
    }
    
    fn colorize_console(&self, level: LogLevel, formatted_msg: &str) -> String {
        match level {
            LogLevel::Debug => formatted_msg.blue().bold().to_string(),
            LogLevel::Info => formatted_msg.green().bold().to_string(),
            LogLevel::Warning => formatted_msg.yellow().bold().to_string(),
            LogLevel::Error => formatted_msg.red().bold().blink().to_string(),
            LogLevel::Critical => formatted_msg.red().bold().blink().to_string(),
        }
    }
    
    fn add_duck_say(&self, level: LogLevel, message: &str) -> String {
        if matches!(level, LogLevel::Error | LogLevel::Critical) {
            let duck_say = format!(
                "\n\x1b[3m\x1b[38;2;0;150;150m🦆 duck say \x1b[1m\x1b[38;2;255;255;0m⮞\x1b[0m\x1b[3m\x1b[38;2;0;150;150m fuck ❌ {}\x1b[0m",
                message
            );
            duck_say
        } else {
            String::new()
        }
    }
    
    pub fn log(&mut self, level: LogLevel, message: &str) {
        if !self.should_log(level) {
            return;
        }
        
        let formatted = self.format_message(level, message);
        let console_output = self.colorize_console(level, &formatted);
        
        eprintln!("{}", console_output);
        
        if matches!(level, LogLevel::Error | LogLevel::Critical) {
            let duck_say = self.add_duck_say(level, message);
            eprintln!("{}", duck_say);
        }
        
        if let Some(file) = &mut self.log_file {
            let timestamp = Local::now().format("%H:%M:%S");
            let level_str = match level {
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warning => "WARNING",
                LogLevel::Error => "ERROR",
                LogLevel::Critical => "CRITICAL",
            };
            
            let file_msg = format!("[{}] {} - {}\n", timestamp, level_str, message);
            let _ = writeln!(file, "{}", file_msg);
        }
    }
}

fn with_logger<F>(level: LogLevel, msg: &str, f: F)
where
    F: FnOnce(&mut DuckTraceLogger, LogLevel, &str),
{
    let logger = LOGGER.get_or_init(|| Mutex::new(DuckTraceLogger::new(None)));
    let mut guard = logger.lock().unwrap();
    f(&mut guard, level, msg);
}

pub fn dt_debug(msg: &str) {
    with_logger(LogLevel::Debug, msg, |logger, lvl, m| logger.log(lvl, m));
}

pub fn dt_info(msg: &str) {
    with_logger(LogLevel::Info, msg, |logger, lvl, m| logger.log(lvl, m));
}

pub fn dt_warning(msg: &str) {
    with_logger(LogLevel::Warning, msg, |logger, lvl, m| logger.log(lvl, m));
}


pub fn dt_error(msg: &str) {
    with_logger(LogLevel::Error, msg, |logger, lvl, m| logger.log(lvl, m));
}

pub fn dt_critical(msg: &str) {
    with_logger(LogLevel::Critical, msg, |logger, lvl, m| logger.log(lvl, m));
}


pub fn dt_setup(_log_name: Option<&str>, level: Option<&str>) {
    let _ = LOGGER.get_or_init(|| Mutex::new(DuckTraceLogger::new(level)));
}

pub struct DtTimer {
    operation_name: String,
    start_time: Instant,
}

impl DtTimer {
    pub fn new(operation_name: &str) -> Self {
        dt_debug(&format!("Starting {}...", operation_name));
        Self {
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
        }
    }
    
    pub fn lap(&self, lap_name: &str) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        dt_debug(&format!("{} - {}: {:.3}s", self.operation_name, lap_name, elapsed));
    }
    
    pub fn complete(self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        dt_debug(&format!("Completed {} in {:.3}s", self.operation_name, elapsed));
    }

}

// Shortcut / Alias
pub fn dt_timer(name: &str) -> DtTimer {
    DtTimer::new(name)
}

#[macro_export]
macro_rules! duck_log {
    (debug: $($arg:tt)*) => {
        $crate::dt_debug(&format!($($arg)*));
    };
    (info: $($arg:tt)*) => {
        $crate::dt_info(&format!($($arg)*));
    };
    (warning: $($arg:tt)*) => {
        $crate::dt_warning(&format!($($arg)*));
    };
    (error: $($arg:tt)*) => {
        $crate::dt_error(&format!($($arg)*));
    };
    (critical: $($arg:tt)*) => {
        $crate::dt_critical(&format!($($arg)*));
    };
}
