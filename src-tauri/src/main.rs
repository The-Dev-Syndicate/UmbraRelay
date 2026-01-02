// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// To enable console window for debugging on Windows, comment out the line below
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Enable backtrace for better error messages in debug builds only
    // In release builds, backtraces are disabled for performance
    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    
    umbra_relay_lib::run()
}
