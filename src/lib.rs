use animation::Frame;

#[cfg(feature = "stack")]
pub mod stack_navigator;

pub mod components {
    pub mod header;
    pub(crate) mod stack_page_wrapper;
}

pub(crate) mod animation;

#[derive(Debug, Clone)]
pub enum NavigationAction<PageMapper> {
    Navigate(PageMapper),
    Tick(Frame),
    GoBack,
}

pub trait Navigator<PageMapper> {
    fn is_on_page(&self, page: PageMapper) -> bool;

    fn is_on_page_and<F: Fn() -> bool>(&self, page: PageMapper, f: F) -> bool;

    fn clear_history(&mut self);
}

pub trait NavigationConvertible {
    type PageMapper;

    fn from_action(action: NavigationAction<Self::PageMapper>) -> Self;
}

pub trait PageComponent<Message> {
    fn view(&self) -> iced::Element<Message>;

    fn update(&mut self, message: Message) -> iced::Task<Message>;

    fn on_load(&self) -> iced::Task<Message> {
        iced::Task::none()
    }
}
