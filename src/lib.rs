use components::header::{HeaderButtonElement, HeaderSettings, HeaderTitleElement};
pub use stack_navigatior::StackNavigator;

mod stack_navigatior;

pub mod components {
    pub mod header;
}

#[derive(Debug, Clone)]
pub enum NavigationAction<P> {
    Navigate(P),
    GoBack,
}

pub trait StackNavigatorMapper {
    type Message: Clone + NavigationConvertible;

    fn title(&self) -> String;

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

pub trait PageComponent<M> {
    fn view(&self) -> iced::Element<M>;

    fn update(&mut self, message: M) -> iced::Task<M>;
}
