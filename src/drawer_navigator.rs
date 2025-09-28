use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use iced::widget::{column, horizontal_space, row};

use crate::{
    components::{
        drawer::{Drawer, DrawerOptionElement, DrawerSettings},
        header::{Header, HeaderButtonElement, HeaderSettings, HeaderTitleElement},
        pages_container::pages_container,
    },
    NavigationAction, NavigationConvertible, Navigator, PageComponent,
};

pub trait DrawerNavigatorMapper: Hash {
    type Message: Clone + NavigationConvertible;

    fn title(&self) -> String;

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>>;

    fn header_settings(&self) -> Option<HeaderSettings> {
        None
    }

    fn settings(&self) -> Option<DrawerSettings> {
        None
    }

    fn back_button(&self) -> Option<Box<dyn HeaderButtonElement<Self::Message>>> {
        None
    }

    fn right_button(&self) -> Option<Box<dyn HeaderButtonElement<Self::Message>>> {
        None
    }

    fn title_widget(&self) -> Option<Box<dyn HeaderTitleElement<Self::Message>>> {
        None
    }

    fn drawer_option<PageMapper>(
        &self,
    ) -> Option<Box<dyn DrawerOptionElement<Self::Message, PageMapper>>>
    where
        PageMapper: DrawerNavigatorMapper + Eq + Clone,
    {
        None
    }

    fn get_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }
}

pub struct DrawerNavigator<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: DrawerNavigatorMapper + Eq + Clone,
{
    current_page: PageMapper,
    drawer: Drawer<Message, PageMapper>,
    pages: HashMap<PageMapper, (u64, Header<Message>, Box<dyn PageComponent<Message>>)>,
    history: Vec<PageMapper>,
}

impl<Message, PageMapper> DrawerNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper>,
    PageMapper: DrawerNavigatorMapper<Message = Message> + Eq + Clone,
{
    pub fn new(
        pages: impl Into<Vec<PageMapper>>,
        initial_page: PageMapper,
    ) -> (Self, iced::Task<Message>) {
        let pages = pages.into();

        let mut navigator = Self {
            current_page: initial_page.clone(),
            history: Vec::with_capacity(pages.len()),
            pages: HashMap::with_capacity(pages.len()),
            drawer: Drawer::new(initial_page.clone(), pages),
        };

        let widget = initial_page.into_component();
        let load_task = widget.on_load();
        let page = navigator.get_page(&initial_page, widget);
        let settings = initial_page.settings();

        navigator.pages.insert(initial_page, page);
        navigator.drawer.set_settings(settings);

        (navigator, load_task)
    }

    pub fn handle_actions(&mut self, message: NavigationAction<PageMapper>) -> iced::Task<Message> {
        match message {
            NavigationAction::Navigate(page) => {
                let mut load_task = iced::Task::none();

                if !self.pages.contains_key(&page) {
                    let widget = page.into_component();
                    load_task = widget.on_load();

                    self.pages
                        .insert(page.clone(), self.get_page(&page, widget));
                }

                self.drawer.set_selected_page(page.clone());

                let old_page = std::mem::replace(&mut self.current_page, page);

                self.history.push(old_page);

                load_task
            }
            NavigationAction::GoBack => {
                if let Some(page) = self.history.pop() {
                    self.drawer.set_selected_page(page.clone());
                    self.current_page = page;
                }

                iced::Task::none()
            }
            NavigationAction::Tick(_) => iced::Task::none(),
        }
    }

    fn get_page(
        &self,
        page: &PageMapper,
        widget: Box<dyn PageComponent<Message>>,
    ) -> (u64, Header<Message>, Box<dyn PageComponent<Message>>) {
        let mut header: Header<Message> = Header::new(page.title());

        header.set_settings(page.header_settings());

        if let Some(button) = page.back_button() {
            header.set_back_button(button);
        }

        if let Some(button) = page.right_button() {
            header.set_right_button(button);
        }

        if let Some(title) = page.title_widget() {
            header.set_title_widget(title);
        }

        (page.get_id(), header, widget)
    }
}

impl<Message, PageMapper> Navigator<PageMapper> for DrawerNavigator<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: DrawerNavigatorMapper<Message = Message> + Eq + Clone,
{
    fn pop_history(&mut self) {
        self.history.pop();
    }

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

impl<Message, PageMapper> PageComponent<Message> for DrawerNavigator<Message, PageMapper>
where
    Message: NavigationConvertible<PageMapper = PageMapper> + Clone,
    PageMapper: DrawerNavigatorMapper<Message = Message> + Clone + Eq,
{
    fn view(&self) -> iced::Element<Message> {
        let (id, header, page) = self
            .pages
            .get(&self.current_page)
            .expect("page should have been initialized");

        let header = if self
            .current_page
            .header_settings()
            .is_none_or(|settings| settings.show_header)
        {
            header.hide_left_button(self.history.is_empty());

            header.view()
        } else {
            horizontal_space().into()
        };

        let container = self
            .history
            .iter()
            .fold(pages_container(), |container, page| {
                let (id, header, widget) = self.pages.get(page).unwrap();

                container
                    .push(*id, column![header.view(), widget.view()])
                    .hide_last(true)
                    .disable_last(true)
            })
            .push(*id, column![header, page.view()])
            .disable_last(false)
            .persist(true);

        row![self.drawer.view(), container].into()
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized")
            .2
            .update(message)
    }
}
