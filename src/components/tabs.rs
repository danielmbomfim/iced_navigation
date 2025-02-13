use iced::{
    widget::{button, text, Row},
    Color,
};
use std::marker::PhantomData;

use crate::{tabs_navigator::TabsNavigatorMapper, NavigationAction, NavigationConvertible};

pub struct Tabs<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper + Eq,
{
    pages: Vec<PageMapper>,
    current_page: PageMapper,
    _message: PhantomData<Message>,
}

impl<Message, PageMapper> Tabs<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper + Eq + Clone,
{
    pub fn new(pages: impl Into<Vec<PageMapper>>, initial_page: PageMapper) -> Self {
        Self {
            pages: pages.into(),
            current_page: initial_page,
            _message: PhantomData,
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        self.pages
            .iter()
            .fold(Row::new(), |row, page| {
                let active_page = self.current_page == *page;

                row.push(
                    button(text(page.title()).color(if active_page {
                        Color::from_rgb(255.0, 0.0, 0.0)
                    } else {
                        Color::BLACK
                    }))
                    .on_press_maybe(if active_page {
                        None
                    } else {
                        Some(Message::from_action(NavigationAction::Navigate(
                            page.clone(),
                        )))
                    }),
                )
            })
            .into()
    }

    pub fn update_current_page(&mut self, page: PageMapper) {
        self.current_page = page;
    }
}
