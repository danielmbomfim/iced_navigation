use std::marker::PhantomData;

use iced::{
    widget::{button, column, container, text, vertical_space},
    Alignment, Background, Color, Element, Length, Padding, Theme,
};
use iced_font_awesome::fa_icon_solid;

use crate::{
    components::header::{ButtonSettings, HeaderButtonElement},
    drawer_navigator::{DrawerAction, DrawerNavigatorMapper},
    NavigationAction, NavigationConvertible,
};

#[derive(Debug, Clone, Copy)]
pub enum DrawerMode {
    Fixed,
    Sliding,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawerSettings {
    pub width: Length,
    pub padding: Padding,
    pub background_color: Option<Color>,
    pub item_settings: DrawerItemsSettings,
    pub mode: DrawerMode,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawerItemsSettings {
    pub active_tint_color: Option<Color>,
    pub inactive_tint_color: Option<Color>,
    pub background_color: Option<Color>,
    pub background_color_selected: Option<Color>,
}

impl Default for DrawerSettings {
    fn default() -> Self {
        Self {
            width: Length::Fixed(300.0),
            padding: Padding::new(10.0).top(60),
            background_color: None,
            item_settings: DrawerItemsSettings::default(),
            mode: DrawerMode::Fixed,
        }
    }
}

impl Default for DrawerItemsSettings {
    fn default() -> Self {
        Self {
            active_tint_color: None,
            inactive_tint_color: None,
            background_color: None,
            background_color_selected: None,
        }
    }
}

pub trait DrawerOptionElement<Message, PageMapper>
where
    Message: Clone + NavigationConvertible,
    PageMapper: DrawerNavigatorMapper + Eq + Clone,
{
    fn view<'a>(
        &'a self,
        page: &PageMapper,
        selected: bool,
        settings: DrawerItemsSettings,
    ) -> iced::Element<'a, Message>
    where
        Message: 'a;
}

pub struct Drawer<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: DrawerNavigatorMapper + Clone,
{
    options: Vec<(
        Box<dyn DrawerOptionElement<Message, PageMapper>>,
        PageMapper,
    )>,
    selected_page: PageMapper,
    settings: DrawerSettings,
    _message: PhantomData<Message>,
}

impl<Message, PageMapper> Drawer<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: DrawerNavigatorMapper<Message = Message> + Eq + Clone,
{
    pub fn new(selected_page: PageMapper, options: Vec<PageMapper>) -> Self {
        Self {
            options: options
                .into_iter()
                .map(|page| {
                    (
                            page.drawer_option().unwrap_or(
                                Box::new(DrawerOption) as Box<dyn DrawerOptionElement<_, _>>
                            ),
                            page,
                        )
                })
                .collect(),
            selected_page,
            settings: DrawerSettings::default(),
            _message: PhantomData,
        }
    }

    pub(crate) fn set_settings(&mut self, settings: Option<DrawerSettings>) {
        self.settings = settings.unwrap_or_else(DrawerSettings::default)
    }

    pub fn set_selected_page(&mut self, selected_page: PageMapper) {
        self.selected_page = selected_page;
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column(self.options.iter().map(|(option, page)| {
                option.view(
                    page,
                    *page == self.selected_page,
                    self.settings.item_settings,
                )
            }))
            .width(self.settings.width)
            .spacing(10)
            .padding(self.settings.padding)
            .align_x(Alignment::Center),
        )
        .style(|theme: &Theme| {
            container::background(
                self.settings
                    .background_color
                    .unwrap_or(theme.palette().primary),
            )
        })
        .height(Length::Fill)
        .into()
    }
}

pub struct DrawerOption;

impl<Message, PageMapper> DrawerOptionElement<Message, PageMapper> for DrawerOption
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper>,
    PageMapper: DrawerNavigatorMapper + Eq + Clone,
{
    fn view<'a>(
        &'a self,
        page: &PageMapper,
        selected: bool,
        settings: DrawerItemsSettings,
    ) -> iced::Element<'a, Message>
    where
        Message: 'a,
    {
        button(text(page.title()).style(move |theme: &iced::Theme| {
            let pallete = theme.extended_palette();

            text::Style {
                color: if selected {
                    settings
                        .active_tint_color
                        .or(Some(pallete.primary.base.text))
                } else {
                    settings
                        .inactive_tint_color
                        .or(Some(pallete.primary.strong.text))
                },
            }
        }))
        .on_press_maybe(if selected {
            None
        } else {
            Some(Message::from_action(NavigationAction::Navigate(
                page.clone(),
            )))
        })
        .width(Length::Fill)
        .style(move |theme: &iced::Theme, status| match status {
            button::Status::Active | button::Status::Pressed => button::Style {
                background: Some(iced::Background::Color(
                    settings.background_color.unwrap_or(theme.palette().primary),
                )),
                ..Default::default()
            },
            button::Status::Hovered | button::Status::Disabled => button::Style {
                background: {
                    Some(iced::Background::Color(
                        settings.background_color_selected.unwrap_or_else(|| {
                            let mut color = theme.extended_palette().primary.strong.color;

                            color.a = 0.6;

                            color
                        }),
                    ))
                },
                ..Default::default()
            },
        })
        .into()
    }
}

pub(crate) struct DrawerButton;

impl<Message> HeaderButtonElement<Message> for DrawerButton
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
            fa_icon_solid("bars")
                .style(move |theme: &iced::Theme| {
                    let pallete = theme.extended_palette();

                    text::Style {
                        color: icon_color.or_else(|| Some(pallete.primary.base.text)),
                    }
                })
                .size(settings.icon_size),
        )
        .on_press(Message::from_action(NavigationAction::Drawer(
            DrawerAction::Expand,
        )))
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

pub fn overlay<'a, Message, PageMapper>() -> Element<'a, Message>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone + 'a,
    PageMapper: DrawerNavigatorMapper<Message = Message> + Eq + Clone + 'a,
{
    button(vertical_space())
        .style(|_, _| button::Style {
            background: Some(Background::Color(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.6,
            })),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press(Message::from_action(NavigationAction::Drawer(
            DrawerAction::Hide,
        )))
        .into()
}
