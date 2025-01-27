use std::{collections::HashMap, hash::Hash, ops::Div};

use iced::{
    widget::{column, container, horizontal_space, Stack},
    Element, Length,
};

use crate::{
    animation::Frame,
    components::{
        header::{Header, HeaderSettings},
        stack_page_wrapper::stack_page_wrapper,
    },
    NavigationAction, NavigationConvertible, PageComponent, StackNavigatorMapper,
};

struct StackNavigatorSettings {
    header_settings: Option<HeaderSettings>,
}

pub struct StackNavigator<M, K>
where
    M: Clone + NavigationConvertible,
    K: Into<Box<dyn PageComponent<M>>> + Eq + Hash + Copy,
{
    current_page: K,
    pages: HashMap<K, (Header<M>, Box<dyn PageComponent<M>>)>,
    history: Vec<K>,
    anim_value: f32,
    transition: bool,
    going_back: bool,
    settings: StackNavigatorSettings,
}

impl<M, K> StackNavigator<M, K>
where
    M: Clone + NavigationConvertible + Send + 'static,
    K: StackNavigatorMapper<Message = M> + Into<Box<dyn PageComponent<M>>> + Eq + Hash + Copy,
{
    pub fn new(initial_page: K) -> Self {
        let mut navigator = Self {
            history: Vec::with_capacity(5),
            current_page: initial_page,
            pages: HashMap::new(),
            anim_value: 0.0,
            going_back: false,
            transition: false,
            settings: StackNavigatorSettings {
                header_settings: None,
            },
        };

        navigator.pages.insert(
            initial_page,
            navigator.get_page(initial_page, initial_page.into()),
        );

        navigator
    }

    pub fn set_header_settings(&mut self, settings: HeaderSettings) {
        self.settings.header_settings = Some(settings);
    }

    pub fn handle_actions(&mut self, message: NavigationAction<K>) -> iced::Task<M> {
        match message {
            NavigationAction::Navigate(page) => {
                if !self.pages.contains_key(&page) {
                    self.pages.insert(page, self.get_page(page, page.into()));
                }

                let old_page = std::mem::replace(&mut self.current_page, page);

                self.history.push(old_page);

                self.start_new_page_animation()
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

                if completed {
                    self.transition = false;

                    if self.going_back {}

                    return iced::Task::none();
                }

                iced::Task::done(M::from_action(NavigationAction::Tick(frame)))
            }
        }
    }

    fn start_new_page_animation(&mut self) -> iced::Task<M> {
        self.anim_value = 0.0;
        self.transition = true;

        iced::Task::done(M::from_action(NavigationAction::Tick(Frame::new())))
    }

    fn start_go_back_animation(&mut self) -> iced::Task<M> {
        self.anim_value = 100.0;
        self.transition = true;

        iced::Task::done(M::from_action(NavigationAction::Tick(
            Frame::new().map(|value| (value - 100.0).abs()),
        )))
    }

    fn get_page(
        &self,
        page: K,
        widget: Box<dyn PageComponent<M>>,
    ) -> (Header<M>, Box<dyn PageComponent<M>>) {
        let mut header: Header<M> = Header::new(page.title());

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

impl<M, K> PageComponent<M> for StackNavigator<M, K>
where
    M: Clone + NavigationConvertible,
    K: Into<Box<dyn PageComponent<M>>> + Eq + Hash + Copy,
{
    fn view(&self) -> iced::Element<M> {
        let (header, page) = self
            .pages
            .get(&self.current_page)
            .expect("page should have been initialized");

        header.hide_left_button(!self.history.is_empty());

        let history: Vec<Element<M>> = self
            .history
            .iter()
            .map(|page| {
                let (header, widget) = self.pages.get(page).unwrap();

                stack_page_wrapper(column![header.view(), widget.view()])
                    .active(false)
                    .into()
            })
            .collect();

        Stack::new()
            .extend(history)
            .push(overlay(self.anim_value))
            .push(
                stack_page_wrapper(column![header.view(), page.view()])
                    .active(!self.transition)
                    .animated(self.transition)
                    .progress(self.anim_value),
            )
            .into()
    }

    fn update(&mut self, message: M) -> iced::Task<M> {
        let (_, page) = self
            .pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized");

        page.update(message)
    }
}

fn overlay<'a, M>(progress: f32) -> iced::Element<'a, M>
where
    M: 'a,
{
    let opacity = progress.div(100.0) * 0.5;

    container(horizontal_space())
        .style(move |_theme| {
            container::background(iced::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: opacity,
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
