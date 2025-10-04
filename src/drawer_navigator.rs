use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use iced::{
    widget::{column, horizontal_space, row},
    Task,
};

use crate::{
    animation::Frame,
    components::{
        drawer::{overlay, Drawer, DrawerButton, DrawerMode, DrawerOptionElement, DrawerSettings},
        header::{Header, HeaderButtonElement, HeaderSettings, HeaderTitleElement},
        pages_container::pages_container,
    },
    NavigationAction, NavigationConvertible, Navigator, PageComponent,
};

#[derive(Debug, Clone, Copy)]
pub enum DrawerAction {
    Hide,
    Expand,
}

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
    anim_value: f32,
    transition: bool,
    show_drawer: bool,
}

impl<Message, PageMapper> DrawerNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible<PageMapper = PageMapper> + Send + 'static,
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
            anim_value: 0.0,
            transition: false,
            show_drawer: false,
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

                if matches![
                    self.current_page.settings().map(|s| s.mode),
                    Some(DrawerMode::Sliding)
                ] {
                    return Task::batch([
                        Task::done(Message::from_action(NavigationAction::Drawer(
                            DrawerAction::Hide,
                        ))),
                        load_task,
                    ]);
                }

                load_task
            }
            NavigationAction::GoBack => {
                if let Some(page) = self.history.pop() {
                    self.drawer.set_selected_page(page.clone());
                    self.current_page = page;
                }

                iced::Task::none()
            }
            NavigationAction::Tick(mut frame) => {
                frame.update();

                self.anim_value = frame.get_value();

                if frame.is_complete() {
                    self.transition = false;

                    self.show_drawer = self.anim_value == 0.0;

                    return iced::Task::none();
                }

                iced::Task::done(Message::from_action(NavigationAction::Tick(frame)))
            }
            NavigationAction::Drawer(action) => match action {
                DrawerAction::Expand => self.start_drawer_open_animation(),
                DrawerAction::Hide => self.start_drawer_hide_animation(),
            },
        }
    }

    fn start_drawer_open_animation(&mut self) -> iced::Task<Message> {
        self.anim_value = -100.0;
        self.transition = true;

        iced::Task::done(Message::from_action(NavigationAction::Tick(
            Frame::new().duration(0.2).map(|value| value - 100.0),
        )))
    }

    fn start_drawer_hide_animation(&mut self) -> iced::Task<Message> {
        self.anim_value = 0.0;
        self.transition = true;

        iced::Task::done(Message::from_action(NavigationAction::Tick(
            Frame::new().duration(0.2).map(|value| value * -1.0),
        )))
    }

    fn get_page(
        &self,
        page: &PageMapper,
        widget: Box<dyn PageComponent<Message>>,
    ) -> (u64, Header<Message>, Box<dyn PageComponent<Message>>) {
        let mut header: Header<Message> = Header::new(page.title());

        if matches![page.settings().map(|s| s.mode), Some(DrawerMode::Sliding)] {
            header.set_back_button(Box::new(DrawerButton));
        }

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
        let mode = self
            .current_page
            .settings()
            .map(|settings| settings.mode)
            .unwrap_or(DrawerMode::Fixed);

        let (id, header, page) = self
            .pages
            .get(&self.current_page)
            .expect("page should have been initialized");

        let header_settings = self.current_page.header_settings();

        let header = if header_settings.is_none_or(|settings| settings.show_header) {
            if matches![
                self.current_page.settings().map(|s| s.mode),
                Some(DrawerMode::Fixed)
            ] {
                header.hide_left_button(self.history.is_empty());
            }

            header.view()
        } else {
            horizontal_space().into()
        };

        let container = self
            .history
            .iter()
            .fold(
                pages_container().persist(true).relative_anim(true),
                |container, page| {
                    let (id, header, widget) = self.pages.get(page).unwrap();

                    container
                        .push(*id, column![header.view(), widget.view()])
                        .hide_last(true)
                        .disable_last(true)
                },
            )
            .push(*id, column![header, page.view()])
            .disable_last(false);

        match mode {
            DrawerMode::Fixed => row![self.drawer.view(), container].into(),
            DrawerMode::Sliding => {
                if self.transition {
                    return container
                        .hide_last(false)
                        .push(0, overlay())
                        .disable_last(true)
                        .push(0, self.drawer.view())
                        .n_progress_last(Some(self.anim_value))
                        .visible_layers(3)
                        .system_layers(2)
                        .no_background_layers(2)
                        .into();
                }

                if self.show_drawer {
                    return container
                        .hide_last(false)
                        .disable_last(true)
                        .push(0, row![self.drawer.view(), overlay()])
                        .system_layers(1)
                        .no_background_layers(1)
                        .into();
                }

                container.into()
            }
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        self.pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized")
            .2
            .update(message)
    }
}
