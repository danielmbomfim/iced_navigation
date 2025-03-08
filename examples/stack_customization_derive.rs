use iced::{Element, Task};
use iced_navigation::{
    navigator_message,
    stack_navigator::{StackNavigator, StackNavigatorMapper},
    NavigationConvertible, PageComponent,
};

#[navigator_message(Page)]
#[derive(Debug, Clone, NavigationConvertible)]
pub enum Message {}

#[derive(Debug, Hash, Eq, PartialEq, Clone, StackNavigatorMapper)]
#[message(Message)]
pub enum Page {
    #[page(
        title = "Page A",
        component = "page_a::PageA::new",
        settings = "page_a::settings",
        title_component = "custom_header_elements::title_widget",
        right_button = "custom_header_elements::right_button",
        back_button = "custom_header_elements::back_button"
    )]
    PageA,
    #[page(
        title = "Page B",
        component = "page_b::PageB::new",
        settings = "page_b::settings",
        title_component = "custom_header_elements::title_widget",
        back_button = "custom_header_elements::back_button"
    )]
    PageB,
}

pub mod custom_header_elements {
    use iced::widget::{button, text};
    use iced_font_awesome::fa_icon_solid;
    use iced_navigation::{
        components::header::{
            ButtonSettings, HeaderButtonElement, HeaderTitleElement, TitleSettings,
        },
        NavigationAction, NavigationConvertible,
    };

    use crate::{Message, Page};

    pub struct CustomHeader;

    impl HeaderTitleElement<Message> for CustomHeader {
        fn view(&self, title: String, settings: &TitleSettings) -> iced::Element<Message> {
            let text_color = settings.title_color;

            text!("Custom title: {}", title)
                .size(settings.title_size)
                .style(move |theme: &iced::Theme| {
                    let pallete = theme.extended_palette();

                    text::Style {
                        color: text_color.or_else(|| Some(pallete.primary.base.text)),
                    }
                })
                .into()
        }
    }

    pub struct CustomBackButton;

    impl<Message> HeaderButtonElement<Message> for CustomBackButton
    where
        Message: Clone + NavigationConvertible,
    {
        fn view<'a>(&'a self, _settings: &ButtonSettings) -> iced::Element<'a, Message>
        where
            Message: 'a,
        {
            button(text("custom back button"))
                .on_press(Message::from_action(NavigationAction::GoBack))
                .into()
        }
    }

    pub struct RighButton;

    impl HeaderButtonElement<Message> for RighButton {
        fn view<'a>(&'a self, settings: &ButtonSettings) -> iced::Element<'a, Message>
        where
            Message: 'a,
        {
            button(fa_icon_solid("rocket").size(settings.icon_size))
                .on_press(Message::from_action(NavigationAction::Navigate(
                    Page::PageB,
                )))
                .into()
        }
    }

    pub fn title_widget() -> Box<dyn HeaderTitleElement<Message>> {
        Box::new(CustomHeader)
    }

    pub fn back_button() -> Box<dyn HeaderButtonElement<Message>> {
        Box::new(CustomBackButton)
    }

    pub fn right_button() -> Box<dyn HeaderButtonElement<Message>> {
        Box::new(RighButton)
    }
}

pub mod page_a {
    use iced::{
        color,
        widget::{column, text},
        Element, Task,
    };
    use iced_navigation::{
        components::header::{ButtonSettings, HeaderSettings, TitleSettings},
        PageComponent,
    };

    use crate::Message;

    pub struct PageA;

    pub fn settings() -> HeaderSettings {
        HeaderSettings {
            background_color: Some(color!(0, 255, 0)),
            title_settings: TitleSettings {
                title_color: Some(color!(255, 255, 255)),
                ..Default::default()
            },
            button_settings: ButtonSettings {
                background_color: Some(color!(0, 139, 139)),
                icon_color: Some(color!(255, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    impl PageA {
        pub fn new() -> Self {
            Self
        }
    }

    impl PageComponent<Message> for PageA {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            column![text(concat!(
                "Wellcome to page A, click on the button on the ",
                "right top corner to move to the next page",
            ))]
            .padding(20)
            .into()
        }
    }
}

pub mod page_b {
    use iced::{color, widget::text, Element, Task};
    use iced_navigation::{
        components::header::{ButtonSettings, HeaderSettings, TitleSettings},
        PageComponent,
    };

    use crate::Message;

    pub struct PageB;

    pub fn settings() -> HeaderSettings {
        HeaderSettings {
            background_color: Some(color!(0, 0, 255)),
            title_settings: TitleSettings {
                title_color: Some(color!(255, 255, 255)),
                ..Default::default()
            },
            button_settings: ButtonSettings {
                background_color: Some(color!(0, 139, 139)),
                icon_color: Some(color!(255, 0, 0)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    impl PageB {
        pub fn new() -> Self {
            Self
        }
    }

    impl PageComponent<Message> for PageB {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            text("Wellcome to page B").into()
        }
    }
}

struct App {
    nav: StackNavigator<Message, Page>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (nav, task) = StackNavigator::new(Page::PageA);

        (Self { nav }, task)
    }

    #[allow(irrefutable_let_patterns)]
    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::NavigationAction(action) = &message {
            return self.nav.handle_actions(action.clone());
        }

        self.nav.update(message)
    }

    fn view(&self) -> Element<Message> {
        self.nav.view()
    }
}

fn main() -> iced::Result {
    iced::application("Stack customization", App::update, App::view).run_with(App::new)
}
