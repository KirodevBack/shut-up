#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod config;
mod platform;
mod state;

use app::App;
use config::Config;

fn main() {
    let config = Config::load_or_create();
    let app = App::init(config);
    platform::run(app);
}
