use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use stack_navigation::StackFrame;

pub mod stack_navigation;

pub mod components {
    pub mod header;
}

#[derive(Debug, Clone, Copy)]
pub enum NavigatorType {
    Stacked,
    Custom,
}

#[derive(Debug, Clone)]
pub enum NavigationAction<M, P>
where
    M: From<NavigationAction<M, P>>,
{
    Navigate(P),
    GoBack,
    _Marker(PhantomData<M>),
}

pub trait PageComponent<M> {
    fn view(&self) -> iced::Element<M>;

    fn update(&mut self, message: M) -> iced::Task<M>;

    fn title(&self) -> Option<String> {
        None
    }

    #[allow(unused_variables)]
    fn on_navigation(&mut self, can_go_back: bool) {}
}

pub struct Navigator<'a, M, K>
where
    M: From<NavigationAction<M, K>> + Clone + 'a,
    K: Into<Box<dyn PageComponent<M>>> + Eq + Hash + Copy + 'a,
{
    navigation_type: NavigatorType,
    current_page: K,
    pages: HashMap<K, Box<dyn PageComponent<M> + 'a>>,
    history: Vec<K>,
}

impl<'a, M, K> Navigator<'a, M, K>
where
    M: From<NavigationAction<M, K>> + Clone + 'a,
    K: Into<Box<dyn PageComponent<M>>> + Eq + Hash + Copy + 'a,
{
    pub fn new(navigation_type: NavigatorType, initial_page: K) -> Self {
        let mut navigator = Self {
            navigation_type,
            pages: HashMap::new(),
            current_page: initial_page,
            history: Vec::new(),
        };

        navigator
            .pages
            .insert(initial_page, navigator.get_page(initial_page.into()));

        navigator
    }

    pub fn handle_actions(&mut self, message: NavigationAction<M, K>) -> iced::Task<M> {
        match message {
            NavigationAction::Navigate(page) => {
                if !self.pages.contains_key(&page) {
                    self.pages.insert(page, self.get_page(page.into()));
                }

                let old_page = std::mem::replace(&mut self.current_page, page);

                self.history.push(old_page);
                self.pages
                    .get_mut(&page)
                    .unwrap()
                    .on_navigation(!self.history.is_empty());

                iced::Task::none()
            }
            NavigationAction::GoBack => {
                if let Some(page) = self.history.pop() {
                    self.current_page = page;
                }

                self.pages
                    .get_mut(&self.current_page)
                    .unwrap()
                    .on_navigation(!self.history.is_empty());

                iced::Task::none()
            }
            NavigationAction::_Marker(_) => iced::Task::none(),
        }
    }

    fn get_page(&self, widget: Box<dyn PageComponent<M>>) -> Box<dyn PageComponent<M> + 'a> {
        match self.navigation_type {
            NavigatorType::Stacked => {
                let mut frame = Box::new(StackFrame::new(widget));
                frame.show_left_button(!self.history.is_empty());

                frame
            }
            NavigatorType::Custom => widget,
        }
    }

    pub fn view(&self) -> iced::Element<M> {
        self.pages
            .get(&self.current_page)
            .expect("page should have been initialized")
            .view()
    }

    pub fn update(&mut self, message: M) -> iced::Task<M> {
        self.pages
            .get_mut(&self.current_page)
            .expect("page should have been initialized")
            .update(message)
    }
}
