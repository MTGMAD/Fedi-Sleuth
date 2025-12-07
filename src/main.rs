// Allow non-snake-case for Dioxus component names (they use PascalCase by convention)
#![allow(non_snake_case)]
// Hide console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

mod app;
mod components;
mod config;
mod models;
mod services;
mod utils;

use app::App;

fn main() {
    // Initialize logging with info level, suppress tao windowing warnings
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module(
            "tao::platform_impl::platform::event_loop",
            log::LevelFilter::Error,
        )
        .init();

    // Launch the Dioxus desktop app
    dioxus_desktop::launch_cfg(
        |cx| cx.render(rsx! { App {} }),
        Config::new().with_window(
            WindowBuilder::new()
                .with_title("Pixelfed Search & Download")
                .with_inner_size(dioxus_desktop::LogicalSize::new(1200, 800))
                .with_min_inner_size(dioxus_desktop::LogicalSize::new(800, 600))
                .with_resizable(true),
        ),
    );
}
