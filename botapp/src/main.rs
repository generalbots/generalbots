#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(feature = "mobile"))]
mod build_info;
#[cfg(not(feature = "mobile"))]
mod desktop;

#[cfg(feature = "mobile")]
fn main() {
    botapp::run();
}

#[cfg(all(not(feature = "mobile"), feature = "gblite"))]
fn main() {
    desktop::app_lite::run();
}

#[cfg(all(not(feature = "mobile"), not(feature = "gblite")))]
fn main() {
    desktop::app::run();
}
