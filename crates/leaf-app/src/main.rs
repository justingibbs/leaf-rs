//! LEAF Application entry point

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    leaf_app_lib::run();
}
