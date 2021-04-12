extern crate fontconfig;
extern crate libc;
extern crate nix;
extern crate x11;

pub mod config;
pub mod context;
pub mod font;
pub mod tray;

mod error_handler;
mod signal_handler;
mod xembed;
