use std::sync::atomic::{AtomicBool, Ordering};

static QUIET_MODE: AtomicBool = AtomicBool::new(false);
static VERBOSE_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_quiet_mode(quiet: bool) {
    QUIET_MODE.store(quiet, Ordering::Relaxed);
}

pub fn set_verbose_mode(verbose: bool) {
    VERBOSE_MODE.store(verbose, Ordering::Relaxed);
}

pub fn is_quiet() -> bool {
    QUIET_MODE.load(Ordering::Relaxed)
}

pub fn is_verbose() -> bool {
    VERBOSE_MODE.load(Ordering::Relaxed)
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if !$crate::logger::is_quiet() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        if $crate::logger::is_verbose() && !$crate::logger::is_quiet() {
            println!("🔍 {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("❌ {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if !$crate::logger::is_quiet() {
            eprintln!("⚠️  {}", format!($($arg)*));
        }
    };
}