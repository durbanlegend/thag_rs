use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new(Verbosity::Normal)); // Default to Normal
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

// pub trait Loggable {
//     fn log(&self, verbosity: Verbosity, message: &str);
// }

#[derive(Loggable)]
pub struct Logger {
    pub level: Verbosity,
}

impl Logger {
    pub fn new(level: Verbosity) -> Self {
        Logger { level }
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.level = verbosity;
    }

    pub fn set_global_verbosity(verbosity: Verbosity) {
        let mut logger = LOGGER.lock().unwrap();
        logger.set_verbosity(verbosity);
    }

    // #[inline]
    pub fn log(&self, level: Verbosity, message: &str) {
        if self.level >= level {
            println!("{}", message);
        }
    }

    pub fn log_verbose(&self, message: &str) {
        self.log(Verbosity::Verbose, message);
    }

    pub fn log_normal(&self, message: &str) {
        self.log(Verbosity::Normal, message);
    }

    pub fn log_quiet(&self, message: &str) {
        self.log(Verbosity::Quiet, message);
    }
}

// #[macro_export]
// macro_rules! log {
//     ($logger:expr, $level:expr, $($arg:tt)*) => {
//         if $logger.level >= $level {
//             println!($($arg)*);
//         }
//     };
// }

// #[macro_export]
// macro_rules! log_it {
//     ($verbosity:expr, $($arg:tt)*) => {
//         $crate::logging::LOGGER.log($verbosity, &format!($($arg)*));
//     };
// }

// #[macro_export]
// macro_rules! log_it {
//   ($verbosity:expr, $($arg:tt)*) => {
//     let guard = $crate::logging::LOGGER.lock().unwrap();
//     guard.log($verbosity, &format!($($arg)*));
//   };
// }

#[macro_export]
macro_rules! log {
  ($verbosity:expr, $($arg:tt)*) => {
    $crate::logging::LOGGER.lock().unwrap().log($verbosity, &format!($($arg)*));
  };
}
