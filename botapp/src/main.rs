#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "mobile")]
mod mobile;
#[cfg(not(feature = "mobile"))]
mod build_info;
#[cfg(not(feature = "mobile"))]
mod desktop;

#[cfg(feature = "mobile")]
fn main() {
    mobile::run();
}

#[cfg(not(feature = "mobile"))]
fn main() {
    desktop::app::run();
}
