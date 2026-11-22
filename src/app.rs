// SPDX-License-Identifier: GPL-3.0-only

use std::rc::Rc;

use cosmic::cosmic_config::ConfigSet;
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::time::{self, Duration, Instant};
use cosmic::iced::widget::{stack, svg};
use cosmic::iced::{Animation, Event, Length, Subscription, event, mouse};
use cosmic::prelude::*;
use cosmic::{theme, widget};

pub struct AppModel {
    core: cosmic::Core,
    hovered: bool,
    icon_transition: Animation<bool>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleMode,
    HoverChanged(bool),
    AnimationTick,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "dev.edfloreshz.ModeToggle";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, (): Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let icon_transition =
            Animation::new(core.system_theme_mode().is_dark).duration(Duration::from_millis(160));
        (
            Self {
                core,
                hovered: false,
                icon_transition,
            },
            Task::none(),
        )
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let moon_opacity = self.icon_transition.interpolate(0.0, 1.0, Instant::now());
        let (width, height) = self.core.applet.suggested_size(true);
        let icon = |name: &'static str, opacity| {
            widget::icon(
                widget::icon::from_name(name)
                    .symbolic(true)
                    .size(width)
                    .into(),
            )
            .width(Length::Fixed(f32::from(width)))
            .height(Length::Fixed(f32::from(height)))
            .class(theme::Svg::Custom(Rc::new(|theme| svg::Style {
                color: Some(theme.cosmic().background(theme.transparent).on.into()),
            })))
            .opacity(opacity)
            .into()
        };
        let icons = stack([
            icon("weather-clear-symbolic", 1.0 - moon_opacity),
            icon("weather-clear-night-symbolic", moon_opacity),
        ]);

        self.core
            .applet
            .button_from_element(icons, true)
            .on_press(Message::ToggleMode)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // The button occupies the entire applet surface.
        let hover = event::listen_with(|event, _, _| match event {
            Event::Mouse(mouse::Event::CursorEntered) => Some(Message::HoverChanged(true)),
            Event::Mouse(mouse::Event::CursorLeft) => Some(Message::HoverChanged(false)),
            _ => None,
        });
        if self.icon_transition.is_animating(Instant::now()) {
            Subscription::batch([
                hover,
                time::every(Duration::from_millis(16)).map(|_| Message::AnimationTick),
            ])
        } else {
            hover
        }
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::HoverChanged(hovered) => {
                self.hovered = hovered;
                self.icon_transition.go_mut(
                    self.core.system_theme_mode().is_dark ^ hovered,
                    Instant::now(),
                );
            }
            Message::AnimationTick => {}
            Message::ToggleMode => {
                if let Err(error) = toggle_mode() {
                    eprintln!("Could not switch COSMIC theme mode: {error}");
                }
            }
        }
        Task::none()
    }

    fn system_theme_mode_update(
        &mut self,
        _: &[&'static str],
        mode: &ThemeMode,
    ) -> Task<cosmic::Action<Message>> {
        self.icon_transition
            .go_mut(mode.is_dark ^ self.hovered, Instant::now());
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

fn toggle_mode() -> Result<(), cosmic::cosmic_config::Error> {
    let config = ThemeMode::config()?;
    // Read the latest setting even if libcosmic's theme notification is still pending.
    let is_dark = match ThemeMode::is_dark(&config) {
        Ok(is_dark) => is_dark,
        Err(error) if !error.is_err() => ThemeMode::default().is_dark,
        Err(error) => return Err(error),
    };

    config.set("is_dark", !is_dark)
}
