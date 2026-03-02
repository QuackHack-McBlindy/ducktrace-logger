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
    path::Path,
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
    log_path: Option<String>,
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
    fn new(file_override: Option<&str>, level_override: Option<&str>) -> Self {
        let level = match level_override {
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

        let (log_file, log_path) = if let Some(path_str) = file_override {
            let path = Path::new(path_str);
            let dir = path
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| {
                    env::var("DT_LOG_PATH").unwrap_or_else(|_| {
                        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                        format!("{}/.config/duckTrace", home)
                    })
                });
            let file = path
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown-script.log".to_string());

            Self::open_log_file(&dir, &file)
        } else {
            let dir = env::var("DT_LOG_PATH").unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.config/duckTrace", home)
            });
            let file = env::var("DT_LOG_FILE").unwrap_or_else(|_| "unknown-script.log".to_string());

            Self::open_log_file(&dir, &file)
        };

        let mut logger = Self {
            level,
            log_file,
            log_path,
        };
        logger.log_config();
        logger
    }

    fn open_log_file(dir: &str, filename: &str) -> (Option<File>, Option<String>) {
        if std::fs::create_dir_all(dir).is_err() {
            return (None, None);
        }

        let full_path = Path::new(dir).join(filename);
        let path_str = full_path.to_string_lossy().into_owned();

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)
        {
            Ok(file) => (Some(file), Some(path_str)),
            Err(_) => (None, None),
        }
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

    fn log_config(&mut self) {
        let log_path_display = self
            .log_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("Logging to file disabled");
        let level_str = match self.level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        };
        self.log(
            LogLevel::Debug,
            &format!(
                "Logger initialized: level={}, file={}",
                level_str, log_path_display
            ),
        );
    }

    fn should_log(&self, msg_level: LogLevel) -> bool {
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

        format!(
            "[🦆📜] [{}] {}{}{} ⮞ {}",
            timestamp, symbol, level_str, symbol, message
        )
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
            format!(
                "\n\x1b[3m\x1b[38;2;0;150;150m🦆 duck say \x1b[1m\x1b[38;2;255;255;0m⮞\x1b[0m\x1b[3m\x1b[38;2;0;150;150m fuck ❌ {}\x1b[0m",
                message
            )
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
    let logger = LOGGER.get_or_init(|| Mutex::new(DuckTraceLogger::new(None, None)));
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

pub fn dt_setup(file_override: Option<&str>, level_override: Option<&str>) {
    let _ = LOGGER.get_or_init(|| Mutex::new(DuckTraceLogger::new(file_override, level_override)));
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

pub fn dt_timer(name: &str) -> DtTimer {
    DtTimer::new(name)
}

pub fn dt_duck_say(message: &str) {
    let teal = Color::TrueColor { r: 0, g: 150, b: 150 };
    let yellow = Color::TrueColor { r: 255, g: 255, b: 0 };

    eprintln!(
        "{}{} {}",
        "🦆 duck say ".italic().color(teal),
        "⮞".bold().color(yellow),
        message.italic().color(teal)
    );
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

#[macro_export]
macro_rules! dt_debug {
    ($($arg:tt)*) => {
        $crate::dt_debug(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! dt_info {
    ($($arg:tt)*) => {
        $crate::dt_info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! dt_warning {
    ($($arg:tt)*) => {
        $crate::dt_warning(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! dt_error {
    ($($arg:tt)*) => {
        $crate::dt_error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! dt_critical {
    ($($arg:tt)*) => {
        $crate::dt_critical(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! duck_say {
    ($($arg:tt)*) => {
        $crate::dt_say(&format!($($arg)*));
    };
}
