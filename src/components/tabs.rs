use iced::{
    widget::{button, container, text, Column, Row},
    Alignment, Color, Length,
};
use iced_font_awesome::FaIcon;
use std::marker::PhantomData;

use crate::{tabs_navigator::TabsNavigatorMapper, NavigationAction, NavigationConvertible};

pub struct TabsSettings {
    pub width: Length,
    pub background_color: Option<Color>,
    pub background_tint_color: Option<Color>,
    pub item_setting: TabItemSetting,
    pub alignment: Alignment,
}

pub struct TabItemSetting {
    pub icon_size: f32,
    pub text_size: f32,
    pub color: Option<Color>,
    pub tint_color: Option<Color>,
    pub horizontal: bool,
}

impl Default for TabsSettings {
    fn default() -> Self {
        Self {
            width: Length::Fill,
            background_color: None,
            background_tint_color: None,
            item_setting: TabItemSetting::default(),
            alignment: Alignment::Center,
        }
    }
}

impl Default for TabItemSetting {
    fn default() -> Self {
        Self {
            icon_size: 15.0,
            text_size: 12.0,
            color: None,
            tint_color: None,
            horizontal: false,
        }
    }
}

pub struct Tabs<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper + Eq,
{
    pages: Vec<PageMapper>,
    current_page: PageMapper,
    settings: TabsSettings,
    _message: PhantomData<Message>,
}

impl<Message, PageMapper> Tabs<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper<Message = Message> + Eq + Clone,
{
    pub fn new(pages: impl Into<Vec<PageMapper>>, initial_page: PageMapper) -> Self {
        Self {
            pages: pages.into(),
            current_page: initial_page,
            settings: TabsSettings::default(),
            _message: PhantomData,
        }
    }

    pub(crate) fn set_settings(&mut self, settings: Option<TabsSettings>) {
        self.settings = settings.unwrap_or_else(TabsSettings::default)
    }

    pub fn view(&self) -> iced::Element<Message> {
        let tabs = self
            .pages
            .iter()
            .fold(Row::new(), |row, page| {
                let active_page = self.current_page == *page;

                let icon = page.icon().or(page.fa_icon().map(|(name, font)| {
                    FaIcon::new(name, font)
                        .size(self.settings.item_setting.icon_size)
                        .style(move |theme: &iced::Theme| {
                            let pallete = theme.extended_palette();

                            text::Style {
                                color: if active_page {
                                    self.settings
                                        .item_setting
                                        .color
                                        .or(Some(pallete.primary.base.text))
                                } else {
                                    self.settings
                                        .item_setting
                                        .tint_color
                                        .or(Some(pallete.primary.base.text.scale_alpha(0.5)))
                                },
                            }
                        })
                        .into()
                }));

                let item_container: iced::Element<Message> =
                    if self.settings.item_setting.horizontal {
                        Row::new()
                            .push_maybe(icon)
                            .push_maybe(page.title().map(|title| {
                                text(title)
                                    .size(self.settings.item_setting.text_size)
                                    .style(move |theme: &iced::Theme| {
                                        let pallete = theme.extended_palette();

                                        text::Style {
                                            color: if active_page {
                                                self.settings
                                                    .item_setting
                                                    .color
                                                    .or(Some(pallete.primary.strong.color))
                                            } else {
                                                self.settings
                                                    .item_setting
                                                    .tint_color
                                                    .or(Some(pallete.primary.weak.color))
                                            },
                                        }
                                    })
                            }))
                            .padding(5)
                            .spacing(5)
                            .align_y(Alignment::Center)
                            .into()
                    } else {
                        Column::new()
                            .push_maybe(icon)
                            .push_maybe(page.title().map(|title| {
                                text(title)
                                    .size(self.settings.item_setting.text_size)
                                    .style(move |theme: &iced::Theme| {
                                        let pallete = theme.extended_palette();

                                        text::Style {
                                            color: if active_page {
                                                self.settings
                                                    .item_setting
                                                    .color
                                                    .or(Some(pallete.background.base.text))
                                            } else {
                                                self.settings
                                                    .item_setting
                                                    .tint_color
                                                    .or(Some(pallete.primary.base.text))
                                            },
                                        }
                                    })
                            }))
                            .padding(5)
                            .spacing(2)
                            .align_x(Alignment::Center)
                            .into()
                    };

                row.push(
                    button(item_container)
                        .style(|theme: &iced::Theme, _status| button::Style {
                            background: Some(iced::Background::Color(
                                self.settings
                                    .background_tint_color
                                    .unwrap_or(theme.palette().primary),
                            )),
                            ..Default::default()
                        })
                        .on_press_maybe(if active_page {
                            None
                        } else {
                            Some(Message::from_action(NavigationAction::Navigate(
                                page.clone(),
                            )))
                        }),
                )
            })
            .spacing(10);

        container(tabs)
            .padding(2)
            .align_x(self.settings.alignment)
            .width(self.settings.width)
            .style(|theme| {
                container::background(
                    self.settings
                        .background_color
                        .unwrap_or(theme.palette().primary),
                )
            })
            .into()
    }

    pub fn update_current_page(&mut self, page: PageMapper) {
        self.current_page = page;
    }
}
