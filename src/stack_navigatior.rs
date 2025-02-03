use std::{collections::HashMap, hash::Hash};

use iced::{
    widget::{column, horizontal_space, Stack},
    Element,
};

use crate::{
    animation::Frame,
    components::{header::Header, stack_page_wrapper::stack_page_wrapper},
    NavigationAction, NavigationConvertible, Navigator, PageComponent, StackNavigatorMapper,
};

pub struct StackNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible,
    PageMapper: Eq + Hash,
{
    current_page: PageMapper,
    pages: HashMap<PageMapper, (Header<Message>, Box<dyn PageComponent<Message>>)>,
    history: Vec<PageMapper>,
    anim_value: f32,
    transition: bool,
    going_back: bool,
    reset_mode: bool,
}

impl<Message, PageMapper> StackNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible + Send + 'static,
    PageMapper: StackNavigatorMapper<Message = Message> + Eq + Hash + Clone,
{
    pub fn new(initial_page: PageMapper) -> (Self, iced::Task<Message>) {
        let mut navigator = Self {
            history: Vec::with_capacity(5),
            current_page: initial_page.clone(),
            pages: HashMap::new(),
            anim_value: 0.0,
            going_back: false,
            transition: false,
            reset_mode: false,
        };

        let widget = initial_page.into_component();
        let load_task = widget.on_load();
        let page = navigator.get_page(&initial_page, widget);

        navigator.pages.insert(initial_page, page);

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

                let old_page = std::mem::replace(&mut self.current_page, page);

                self.history.push(old_page);

                iced::Task::batch([self.start_new_page_animation(), load_task])
            }
            NavigationAction::GoBack => {
                self.going_back = true;
                self.start_go_back_animation()
            }
            NavigationAction::Tick(mut frame) => {
                frame.update();

                self.anim_value = frame.get_value();
                let completed = frame.is_complete();

                if completed && self.going_back {
                    self.going_back = false;

                    if let Some(page) = self.history.pop() {
                        self.current_page = page;
                    }
                }

                if completed && self.reset_mode {
                    self.reset_mode = false;
                    self.history.clear();
                }

                if completed {
                    self.transition = false;

                    if self.going_back {}

                    return iced::Task::none();
                }

                iced::Task::done(Message::from_action(NavigationAction::Tick(frame)))
            }
        }
    }

    fn start_new_page_animation(&mut self) -> iced::Task<Message> {
        self.anim_value = 0.0;
        self.transition = true;

        iced::Task::done(Message::from_action(NavigationAction::Tick(Frame::new())))
    }

    fn start_go_back_animation(&mut self) -> iced::Task<Message> {
        self.anim_value = 100.0;
        self.transition = true;

        iced::Task::done(Message::from_action(NavigationAction::Tick(
            Frame::new().map(|value| (value - 100.0).abs()),
        )))
    }

    fn get_page(
        &self,
        page: &PageMapper,
        widget: Box<dyn PageComponent<Message>>,
    ) -> (Header<Message>, Box<dyn PageComponent<Message>>) {
        let mut header: Header<Message> = Header::new(page.title());

        header.set_settings(page.settings());

        if let Some(button) = page.back_button() {
            header.set_back_button(button);
        }

        if let Some(button) = page.right_button() {
            header.set_right_button(button);
        }

        if let Some(title) = page.title_widget() {
            header.set_title_widget(title);
        }

        (header, widget)
    }
}

impl<Message, PageMapper> Navigator<PageMapper> for StackNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible + Send + 'static,
    PageMapper: StackNavigatorMapper<Message = Message> + Eq + Hash,
{
    fn clear_history(&mut self) {
        self.reset_mode = true;
    }

    fn is_on_page(&self, page: PageMapper) -> bool {
        self.current_page == page
    }

    fn is_on_page_and<F: Fn() -> bool>(&self, page: PageMapper, f: F) -> bool {
        self.current_page == page && f()
    }
}

impl<Message, PageMapper> PageComponent<Message> for StackNavigator<Message, PageMapper>
where
    Message: Clone + NavigationConvertible,
    PageMapper: StackNavigatorMapper<Message = Message> + Eq + Hash,
{
    fn view(&self) -> iced::Element<Message> {
        let (header, page) = self
            .pages
            .get(&self.current_page)
            .expect("page should have been initialized");

        let header = if self
            .current_page
            .settings()
            .is_none_or(|settings| settings.show_header)
        {
            header.hide_left_button(self.reset_mode || self.history.is_empty());

            header.view()
        } else {
            horizontal_space().into()
        };

        let previous_pages: Vec<Element<_>> = self
            .history
            .iter()
            .map(|page| {
                let (header, widget) = self.pages.get(page).unwrap();

                let header = if page.settings().is_none_or(|settings| settings.show_header) {
                    header.view()
                } else {
                    horizontal_space().into()
                };

                stack_page_wrapper(column![header, widget.view()])
                    .active(false)
                    .hide(!self.transition)
                    .animated(self.transition)
                    .n_progress(self.anim_value * -0.4)
                    .into()
            })
            .collect();

        Stack::new()
            .extend(previous_pages)
            .push(
                stack_page_wrapper(column![header, page.view()])
                    .active(!self.transition)
                    .animated(self.transition)
                    .progress(self.anim_value),
            )
            .into()
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        let (_, page) = self
            .pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized");

        page.update(message)
    }
}
