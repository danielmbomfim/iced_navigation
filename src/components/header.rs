use std::cell::RefCell;

use iced::{
    widget::{button, container, horizontal_space, text, Row},
    Alignment, Color, Length, Pixels,
};
use iced_font_awesome::fa_icon_solid;

use crate::{NavigationAction, NavigationConvertible};

pub struct HeaderSettings {
    pub show_header: bool,
    pub height: Length,
    pub background_color: Option<Color>,
    pub button_settings: ButtonSettings,
    pub title_settings: TitleSettings,
}

pub struct ButtonSettings {
    pub background_color: Option<Color>,
    pub height: Length,
    pub width: Length,
    pub icon_color: Option<Color>,
    pub icon_size: f32,
}

pub struct TitleSettings {
    pub title_color: Option<Color>,
    pub title_size: Pixels,
}

impl Default for HeaderSettings {
    fn default() -> Self {
        Self {
            show_header: true,
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
            icon_color: None,
            icon_size: 20.0,
            background_color: None,
        }
    }
}

pub trait HeaderButtonElement<Message>
where
    Message: Clone + NavigationConvertible,
{
    fn view<'a>(&'a self, settings: &ButtonSettings) -> iced::Element<'a, Message>
    where
        Message: 'a;
}

pub trait HeaderTitleElement<Message> {
    fn view(&self, title: String, settings: &TitleSettings) -> iced::Element<Message>;
}

pub struct Header<Message> {
    title: String,
    title_widget: Box<dyn HeaderTitleElement<Message>>,
    back_button: Box<dyn HeaderButtonElement<Message>>,
    right_button: Option<Box<dyn HeaderButtonElement<Message>>>,
    settings: HeaderSettings,
    show_left_button: RefCell<bool>,
}

impl<Message> Header<Message>
where
    Message: Clone + NavigationConvertible,
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

        *value = !hide;
    }

    pub fn set_back_button(&mut self, button: Box<dyn HeaderButtonElement<Message>>) {
        self.back_button = button;
    }

    pub fn set_right_button(&mut self, button: Box<dyn HeaderButtonElement<Message>>) {
        self.right_button = Some(button);
    }

    pub fn set_title_widget(&mut self, title: Box<dyn HeaderTitleElement<Message>>) {
        self.title_widget = title;
    }

    fn render_back_button(&self) -> Option<iced::Element<Message>> {
        if !*self.show_left_button.borrow() {
            return None;
        }

        Some(self.back_button.view(&self.settings.button_settings))
    }

    pub fn view(&self) -> iced::Element<Message> {
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
        .style(|theme| {
            container::background(
                self.settings
                    .background_color
                    .unwrap_or(theme.palette().primary),
            )
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

impl<Message> HeaderTitleElement<Message> for Title {
    fn view(&self, title: String, settings: &TitleSettings) -> iced::Element<Message> {
        let text_color = settings.title_color;

        text(title)
            .size(settings.title_size)
            .style(move |theme: &iced::Theme| {
                let pallete = theme.extended_palette();

                text::Style {
                    color: text_color.or_else(|| Some(pallete.primary.base.text)),
                }
            })
            .into()
    }
}

pub struct BackButton;

impl BackButton {
    pub fn new() -> Self {
        Self
    }
}

impl<Message> HeaderButtonElement<Message> for BackButton
where
    Message: Clone + NavigationConvertible,
{
    fn view<'a>(&'a self, settings: &ButtonSettings) -> iced::Element<'a, Message>
    where
        Message: 'a,
    {
        let background = settings.background_color;
        let icon_color = settings.icon_color;

        button(
            fa_icon_solid("angle-left")
                .style(move |theme: &iced::Theme| {
                    let pallete = theme.extended_palette();

                    text::Style {
                        color: icon_color.or_else(|| Some(pallete.primary.base.text)),
                    }
                })
                .size(settings.icon_size),
        )
        .on_press(Message::from_action(NavigationAction::GoBack))
        .style(move |theme: &iced::Theme, status| match status {
            button::Status::Active | button::Status::Pressed => button::Style {
                background: Some(iced::Background::Color(
                    background.unwrap_or(theme.palette().primary),
                )),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: {
                    let mut color = background.unwrap_or(theme.palette().primary);

                    color.a = 0.6;

                    Some(iced::Background::Color(color))
                },
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: {
                    let mut color = background.unwrap_or(theme.palette().primary);

                    color.a = 0.3;

                    Some(iced::Background::Color(color))
                },
                ..Default::default()
            },
        })
        .width(settings.width)
        .height(settings.height)
        .into()
    }
}
