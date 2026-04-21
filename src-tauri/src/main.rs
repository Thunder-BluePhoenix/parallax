// Parallax - Rust Tauri Backend Entry Point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    parallax_lib::run();
}
