use iced::widget::column;
use iced_font_awesome::IconFont;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

#[cfg(feature = "derive")]
pub use iced_navigation_derive::TabsNavigatorMapper;

use crate::{
    components::{
        pages_container::pages_container,
        tabs::{Tabs, TabsSettings},
    },
    NavigationAction, NavigationConvertible, Navigator, PageComponent,
};

pub trait TabsNavigatorMapper: Hash {
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

    fn get_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }
}

pub enum Position {
    Top,
    Bottom,
}

pub struct TabsNavigator<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: TabsNavigatorMapper + Clone + Eq,
{
    position: Position,
    current_page: PageMapper,
    pages: HashMap<PageMapper, (u64, Box<dyn PageComponent<Message>>)>,
    history: Vec<PageMapper>,
    tabs: Tabs<Message, PageMapper>,
}

impl<Message, PageMapper> TabsNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper>,
    PageMapper: TabsNavigatorMapper<Message = Message> + Eq + Clone,
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

        let hashed_page = (initial_page.get_id(), widget);

        navigator.pages.insert(initial_page, hashed_page);
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

                    self.pages.insert(page.clone(), (page.get_id(), widget));
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
            #[cfg(feature = "drawer")]
            NavigationAction::Drawer(_) => iced::Task::none(),
        }
    }
}

impl<Message, PageMapper> Navigator<PageMapper> for TabsNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper>,
    PageMapper: TabsNavigatorMapper<Message = Message> + Eq + Clone,
{
    fn pop_history(&mut self) {}

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
    PageMapper: TabsNavigatorMapper<Message = Message> + Clone + Eq,
{
    fn view(&self) -> iced::Element<Message> {
        let (id, page) = self
            .pages
            .get(&self.current_page)
            .expect("page should have been initialized");

        let container = self
            .history
            .iter()
            .fold(pages_container(), |container, page| {
                let (id, widget) = self.pages.get(page).unwrap();

                container
                    .push(*id, widget.view())
                    .disable_last(true)
                    .hide_last(true)
            })
            .push(*id, page.view())
            .disable_last(false)
            .persist(true);

        match self.position {
            Position::Top => column![self.tabs.view(), container].into(),
            Position::Bottom => column![container, self.tabs.view()].into(),
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized")
            .1
            .update(message)
    }
}
