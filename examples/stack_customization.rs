use iced::{color, Element, Task};
use iced_navigation::{
    components::header::{
        ButtonSettings, HeaderButtonElement, HeaderSettings, HeaderTitleElement, TitleSettings,
    },
    stack_navigator::{StackNavigator, StackNavigatorMapper},
    NavigationAction, NavigationConvertible, PageComponent,
};

#[derive(Debug, Clone)]
enum Message {
    NavigationAction(NavigationAction<Page>),
}

impl NavigationConvertible for Message {
    type PageMapper = Page;

    fn from_action(action: NavigationAction<Self::PageMapper>) -> Self {
        Self::NavigationAction(action)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Page {
    PageA,
    PageB,
}

impl StackNavigatorMapper for Page {
    type Message = Message;

    fn title(&self) -> String {
        match self {
            Page::PageA => "Page A".to_owned(),
            Page::PageB => "Page B".to_owned(),
        }
    }

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
        match self {
            Page::PageA => Box::new(page_a::PageA),
            Page::PageB => Box::new(page_b::PageB),
        }
    }

    fn settings(&self) -> Option<HeaderSettings> {
        let mut settings = HeaderSettings {
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
        };

        match self {
            Page::PageA => settings.background_color = Some(color!(0, 255, 0)),
            _ => {}
        };

        Some(settings)
    }

    fn back_button(&self) -> Option<Box<dyn HeaderButtonElement<Self::Message>>> {
        Some(Box::new(custom_header_elements::CustomBackButton))
    }

    fn right_button(&self) -> Option<Box<dyn HeaderButtonElement<Self::Message>>> {
        match self {
            Page::PageA => Some(Box::new(custom_header_elements::RighButton)),
            Page::PageB => None,
        }
    }

    fn title_widget(&self) -> Option<Box<dyn HeaderTitleElement<Self::Message>>> {
        Some(Box::new(custom_header_elements::CustomHeader))
    }
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

    use crate::Page;

    pub struct CustomHeader;

    impl<Message> HeaderTitleElement<Message> for CustomHeader {
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

    impl<Message> HeaderButtonElement<Message> for RighButton
    where
        Message: Clone + NavigationConvertible<PageMapper = Page>,
    {
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
}

pub mod page_a {
    use iced::{
        widget::{column, text},
        Element, Task,
    };
    use iced_navigation::PageComponent;

    use crate::Message;

    pub struct PageA;

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
    use iced::{widget::text, Element, Task};
    use iced_navigation::PageComponent;

    use crate::Message;

    pub struct PageB;

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
    iced::application("Stack login example", App::update, App::view).run_with(App::new)
}
