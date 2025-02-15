use iced::{
    widget::{column, container},
    Length,
};
use iced_font_awesome::IconFont;
use std::{collections::HashMap, hash::Hash};

use crate::{
    components::tabs::{Tabs, TabsSettings},
    NavigationAction, NavigationConvertible, Navigator, PageComponent,
};

pub trait TabsNavigatorMapper {
    type Message: Clone + NavigationConvertible;

    fn title(&self) -> Option<String> {
        None
    }

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>>;

    fn icon(&self) -> Option<iced::Element<Self::Message>> {
        None
    }

    fn fa_icon(&self) -> Option<(&str, IconFont)> {
        Some(("font-awesome", IconFont::Solid))
    }

    fn settings(&self) -> Option<TabsSettings> {
        None
    }
}

pub enum Position {
    Top,
    Bottom,
}

pub struct TabsNavigator<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper + Clone + Eq + Hash,
{
    position: Position,
    current_page: PageMapper,
    pages: HashMap<PageMapper, Box<dyn PageComponent<Message>>>,
    history: Vec<PageMapper>,
    tabs: Tabs<Message, PageMapper>,
}

impl<Message, PageMapper> TabsNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper>,
    PageMapper: TabsNavigatorMapper<Message = Message> + Eq + Hash + Clone,
{
    pub fn new(
        pages: impl Into<Vec<PageMapper>>,
        initial_page: PageMapper,
    ) -> (Self, iced::Task<Message>) {
        let mut navigator = Self {
            tabs: Tabs::new(pages, initial_page.clone()),
            position: Position::Bottom,
            current_page: initial_page.clone(),
            pages: HashMap::new(),
            history: Vec::new(),
        };

        let widget = initial_page.into_component();
        let settings = initial_page.settings();
        let load_task = widget.on_load();

        navigator.pages.insert(initial_page, widget);
        navigator.tabs.set_settings(settings);

        (navigator, load_task)
    }

    pub fn set_tabs_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn handle_actions(&mut self, message: NavigationAction<PageMapper>) -> iced::Task<Message> {
        match message {
            NavigationAction::Navigate(page) => {
                let mut load_task = iced::Task::none();
                let settings = page.settings();

                if !self.pages.contains_key(&page) {
                    let widget = page.into_component();

                    load_task = widget.on_load();

                    self.pages.insert(page.clone(), widget);
                }

                self.tabs.update_current_page(page.clone());
                let old_page = std::mem::replace(&mut self.current_page, page);

                self.tabs.set_settings(settings);

                self.history.push(old_page);

                load_task
            }
            NavigationAction::GoBack => {
                if let Some(page) = self.history.pop() {
                    self.current_page = page;
                }

                iced::Task::none()
            }
            NavigationAction::Tick(_) => iced::Task::none(),
        }
    }
}

impl<Message, PageMapper> Navigator<PageMapper> for TabsNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper>,
    PageMapper: TabsNavigatorMapper<Message = Message> + Eq + Hash + Clone,
{
    fn clear_history(&mut self) {
        self.history.clear();
    }

    fn is_on_page(&self, page: PageMapper) -> bool {
        self.current_page == page
    }

    fn is_on_page_and<F: Fn() -> bool>(&self, page: PageMapper, f: F) -> bool {
        self.current_page == page && f()
    }
}

impl<Message, PageMapper> PageComponent<Message> for TabsNavigator<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper<Message = Message> + Clone + Eq + Hash,
{
    fn view(&self) -> iced::Element<Message> {
        let page = container(
            self.pages
                .get(&self.current_page)
                .expect("page should have been initialized")
                .view(),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        match self.position {
            Position::Top => column![self.tabs.view(), page].into(),
            Position::Bottom => column![page, self.tabs.view()].into(),
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized")
            .update(message)
    }
}
