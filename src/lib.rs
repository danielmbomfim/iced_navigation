use animation::Frame;
use components::header::{HeaderButtonElement, HeaderSettings, HeaderTitleElement};
pub use stack_navigatior::StackNavigator;

mod stack_navigatior;

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

pub trait StackNavigatorMapper {
    type Message: Clone + NavigationConvertible;

    fn title(&self) -> String;

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>>;

    fn settings(&self) -> Option<HeaderSettings> {
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
