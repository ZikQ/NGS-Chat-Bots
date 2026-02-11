#![windows_subsystem = "windows"]

use crate::app::App;

mod twitch_utils;
mod app;

fn main() {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .exit_on_close_request(true)
        .run();
}