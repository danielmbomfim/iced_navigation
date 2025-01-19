use std::cell::RefCell;

use iced::{
    widget::{button, container, horizontal_space, text, Row},
    Alignment, Color, Length, Pixels,
};
use iced_font_awesome::fa_icon_solid;

use crate::{NavigationAction, NavigationConvertible};

pub struct HeaderSettings {
    pub height: Length,
    pub background_color: Option<Color>,
    pub button_settings: ButtonSettings,
    pub title_settings: TitleSettings,
}

pub struct ButtonSettings {
    pub background_color: Option<Color>,
    pub height: Length,
    pub width: Length,
    pub icon_color: Color,
    pub icon_size: f32,
}

pub struct TitleSettings {
    pub title_color: Option<Color>,
    pub title_size: Pixels,
}

impl Default for HeaderSettings {
    fn default() -> Self {
        Self {
            height: Length::from(50),
            background_color: None,
            button_settings: ButtonSettings::default(),
            title_settings: TitleSettings::default(),
        }
    }
}

impl Default for TitleSettings {
    fn default() -> Self {
        Self {
            title_color: None,
            title_size: Pixels::from(18),
        }
    }
}

impl Default for ButtonSettings {
    fn default() -> Self {
        Self {
            height: Length::from(30),
            width: Length::from(40),
            icon_color: Color::WHITE,
            icon_size: 20.0,
            background_color: None,
        }
    }
}

pub trait HeaderButtonElement<M>
where
    M: Clone + NavigationConvertible,
{
    fn view<'a>(&'a self, settings: &ButtonSettings) -> iced::Element<'a, M>
    where
        M: 'a;
}

pub trait HeaderTitleElement<M> {
    fn view(&self, title: String, settings: &TitleSettings) -> iced::Element<M>;
}

pub struct Header<M> {
    title: String,
    title_widget: Box<dyn HeaderTitleElement<M>>,
    back_button: Box<dyn HeaderButtonElement<M>>,
    right_button: Option<Box<dyn HeaderButtonElement<M>>>,
    settings: HeaderSettings,
    show_left_button: RefCell<bool>,
}

impl<M> Header<M>
where
    M: Clone + NavigationConvertible,
{
    pub fn new(title: String) -> Self {
        Self {
            title,
            title_widget: Box::new(Title::new()),
            back_button: Box::new(BackButton::new()),
            right_button: None,
            settings: HeaderSettings::default(),
            show_left_button: RefCell::new(true),
        }
    }

    pub fn set_settings(&mut self, settings: Option<HeaderSettings>) {
        self.settings = settings.unwrap_or_else(HeaderSettings::default);
    }

    pub fn hide_left_button(&self, hide: bool) {
        let mut value = self.show_left_button.borrow_mut();

        *value = hide;
    }

    pub fn set_back_button(&mut self, button: Box<dyn HeaderButtonElement<M>>) {
        self.back_button = button;
    }

    pub fn set_right_button(&mut self, button: Box<dyn HeaderButtonElement<M>>) {
        self.right_button = Some(button);
    }

    pub fn set_title_widget(&mut self, title: Box<dyn HeaderTitleElement<M>>) {
        self.title_widget = title;
    }

    fn render_back_button(&self) -> Option<iced::Element<M>> {
        if !*self.show_left_button.borrow() {
            return None;
        }

        Some(self.back_button.view(&self.settings.button_settings))
    }

    pub fn view(&self) -> iced::Element<M> {
        container(
            Row::new()
                .push_maybe(self.render_back_button())
                .push(
                    self.title_widget
                        .view(self.title.clone(), &self.settings.title_settings),
                )
                .push(horizontal_space())
                .push_maybe(
                    self.right_button
                        .as_ref()
                        .map(|button| button.view(&self.settings.button_settings)),
                )
                .spacing(20)
                .padding(10)
                .width(Length::Fill)
                .height(self.settings.height)
                .align_y(Alignment::Center),
        )
        .style(|_style| container::Style {
            background: self.settings.background_color.map(iced::Background::Color),
            ..Default::default()
        })
        .into()
    }
}

pub struct Title;

impl Title {
    pub fn new() -> Self {
        Self
    }
}

impl<M> HeaderTitleElement<M> for Title {
    fn view(&self, title: String, settings: &TitleSettings) -> iced::Element<M> {
        text(title)
            .size(settings.title_size)
            .color_maybe(settings.title_color)
            .into()
    }
}

pub struct BackButton;

impl BackButton {
    pub fn new() -> Self {
        Self
    }
}

impl<M> HeaderButtonElement<M> for BackButton
where
    M: Clone + NavigationConvertible,
{
    fn view<'a>(&'a self, settings: &ButtonSettings) -> iced::Element<'a, M>
    where
        M: 'a,
    {
        let background = settings.background_color;

        button(
            fa_icon_solid("angle-left")
                .color(settings.icon_color)
                .size(settings.icon_size),
        )
        .on_press(M::from_action(NavigationAction::GoBack))
        .style(move |_theme, status| match status {
            button::Status::Active | button::Status::Pressed => button::Style {
                background: background.map(iced::Background::Color),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: background.map(|mut color| {
                    color.a = 0.6;

                    iced::Background::Color(color)
                }),
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: background.map(|mut color| {
                    color.a = 0.3;

                    iced::Background::Color(color)
                }),
                ..Default::default()
            },
        })
        .width(settings.width)
        .height(settings.height)
        .into()
    }
}
