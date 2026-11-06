// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod data;
mod logic;
mod update;
mod view;

use std::time::Duration;

use data::{AppTheme, FileReadLoop, Message, RScashM};
use iced::{Size, Subscription, Theme, window};

// Keep retrying the startup read until the ledger is ready.
fn subscription(state: &RScashM) -> Subscription<Message> {
    match state.read_state {
        FileReadLoop::Loaded(_) => Subscription::none(),
        _ => iced::time::every(Duration::from_millis(500)).map(|_| Message::FileReadTick),
    }
}

// Use the palette currently selected in Settings.
fn theme(state: &RScashM) -> Theme {
    match state.app_theme {
        AppTheme::Dark => Theme::Dark,
        AppTheme::Light => Theme::Light,
    }
}

// Set useful window limits and a stable Linux app ID.
fn window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(900.0, 650.0),
        min_size: Some(Size::new(480.0, 360.0)),
        max_size: Some(Size::new(1600.0, 1000.0)),
        resizable: true,
        platform_specific: window::settings::PlatformSpecific {
            application_id: "rscashm".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// Start the iced app with its view, update loop, and window settings.
fn main() -> iced::Result {
    iced::application(RScashM::new, RScashM::update, RScashM::view)
        .title("RScashM")
        .subscription(subscription)
        .theme(theme)
        .window(window_settings())
        .run()
}
