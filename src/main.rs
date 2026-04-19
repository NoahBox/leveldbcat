#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod config;
mod i18n;
mod reader;
mod widgets;

fn main() {
    app::run();
}
