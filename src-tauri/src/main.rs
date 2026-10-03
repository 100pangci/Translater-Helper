// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod linux_compat;

fn main() {
    // WebKit reads renderer settings during initialization, before the window exists.
    #[cfg(target_os = "linux")]
    linux_compat::configure_webkit();

    transhelper_lib::run()
}
