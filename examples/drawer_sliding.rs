#[cfg(feature = "drawer")]
mod article_page {
    use iced::{
        widget::{column, text},
        Element, Task,
    };
    use iced_navigation::PageComponent;

    use crate::app::Message;

    pub struct Page;

    impl PageComponent<Message> for Page {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            column![
                  text("Article\n").size(30),
                  text(concat!(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla mattis diam quis purus vulputate, convallis imperdiet tortor dapibus. ",
                    "Aliquam erat volutpat. Proin a fringilla est. Donec nulla dolor, ornare ac risus at, hendrerit dapibus nunc. Pellentesque nec iaculis ",
                    " nisl. Aliquam aliquam efficitur nisi, et tincidunt justo dictum non. Nam id dui ut purus sodales lacinia.\n"
                  )), text(concat!(
                    "Nam id rutrum justo, non volutpat dolor. Morbi vel sem eu sapien consequat ultrices nec vitae lorem. ",
                    "Mauris consequat leo libero, nec semper lorem tincidunt quis. Donec sed orci metus. Aenean viverra at odio sit amet auctor. ",
                    "Mauris eleifend iaculis molestie. Donec fringilla mi eget justo pellentesque, ut dapibus lorem iaculis. Nullam quis mattis nunc, ",
                    "et placerat erat. Mauris eget dignissim orci, sit amet scelerisque ante.\n"
                  )), text(concat!(
                    "Sed mi lacus, euismod et placerat at, imperdiet ut quam. Phasellus pretium odio id commodo vestibulum. ",
                    "Praesent aliquet vitae orci a sollicitudin. Vestibulum facilisis metus sit amet magna scelerisque malesuada. ",
                    "Maecenas gravida ac metus et semper. Ut sit amet dui ut tortor elementum pretium. Ut leo purus, tincidunt dapibus est vulputate, ",
                    "ultrices imperdiet urna. Phasellus fringilla, mauris ac accumsan aliquam, turpis risus laoreet ipsum, eget porttitor mi odio et nibh. ",
                    "In luctus, lorem vel suscipit sollicitudin, tortor sapien feugiat sapien, eget mollis ante lectus a ipsum.",
                    " Sed imperdiet ullamcorper diam et dignissim. Donec dapibus lorem a est feugiat, eget imperdiet tortor consequat. ",
                    "Curabitur ligula elit, blandit vel blandit vitae, semper vel metus. Nulla sed ex odio. Morbi maximus elit a odio luctus, a mollis purus interdum.\n"
                    )),
                ].padding(20).into()
        }
    }
}

#[cfg(feature = "drawer")]
mod list_page {
    use iced::{
        widget::{row, scrollable, text, Column},
        Element, Length, Task,
    };
    use iced_navigation::PageComponent;

    use crate::app::Message;

    pub struct Page;

    static NAMES: [&str; 25] = [
        "Zoe Frederico Göbel",
        "Lúcio Christoffer Arnold",
        "Walter Pasquale Hennig",
        "Brónach Orquídea Beyer",
        "Kōsuke Adamo Brinkerhoff",
        "Evandro Gudrun Spitz",
        "Katsu Isabelle Casale",
        "Yūma Loreta Pace",
        "Amelia Victor Kilpatrick",
        "Gilberto Björn Alesini",
        "Eva Martina Goebel",
        "Amalia Haru Alfero",
        "Conley Rúben Valenti",
        "Shin Iris Ahlgren",
        "Kyoko Agata Pacheco",
        "Bárbara Niklaus Buonarroti",
        "Kennet Reynaldo Persson",
        "Klara Diodato Okazaki",
        "Saskia Candido Ó Domhnaill",
        "Irene Hipólito Mizushima",
        "Julian Keeva Antunes",
        "Dagobert Ryo Como",
        "Monika Kazuhiko Alinari",
        "Maximilian Epifanio Nuremberg",
        "Artemio Emi Andrade",
    ];

    impl PageComponent<Message> for Page {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            scrollable(
                NAMES
                    .iter()
                    .enumerate()
                    .fold(
                        Column::new().push(text("Members\n").size(30)),
                        |column, (index, name)| {
                            column.push(
                                row![text!("{}", index + 1).size(20), text!("{}", name).size(20)]
                                    .spacing(10),
                            )
                        },
                    )
                    .width(Length::Fill)
                    .padding(20)
                    .spacing(10),
            )
            .into()
        }
    }
}

#[cfg(feature = "drawer")]
mod settings_page {
    use iced::{
        widget::{column, container, text},
        Alignment, Element, Length, Task,
    };
    use iced_font_awesome::fa_icon_solid;
    use iced_navigation::PageComponent;

    use crate::app::Message;

    pub struct Page;

    impl PageComponent<Message> for Page {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            column![
                text("Profile\n").size(30),
                container(fa_icon_solid("circle-user").size(200.0).style(|theme: &iced::Theme| {
                  let palette = theme.extended_palette();

                  text::Style {
                    color: Some(palette.background.base.text)
                  }
                }))
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(30),
                text("name: ------------").size(20),
                text(concat!(
                  "Nam id rutrum justo, non volutpat dolor. Morbi vel sem eu sapien consequat ultrices nec vitae lorem. ",
                  "Mauris consequat leo libero, nec semper lorem tincidunt quis. Donec sed orci metus. Aenean viverra at odio sit amet auctor. ",
                  "Mauris eleifend iaculis molestie. Donec fringilla mi eget justo pellentesque, ut dapibus lorem iaculis. Nullam quis mattis nunc, ",
                  "et placerat erat. Mauris eget dignissim orci, sit amet scelerisque ante.\n"
                )).size(20),
            ]
            .spacing(10)
            .padding(20)
            .into()
        }
    }
}

#[cfg(feature = "drawer")]
mod app {
    use iced::{Element, Task};
    use iced_navigation::{
        components::drawer::{DrawerMode, DrawerSettings},
        drawer_navigator::{DrawerNavigator, DrawerNavigatorMapper},
        NavigationAction, NavigationConvertible, PageComponent,
    };

    #[derive(Debug, Clone)]
    pub enum Message {
        NavigationAction(NavigationAction<Page>),
    }

    impl NavigationConvertible for Message {
        type PageMapper = Page;

        fn from_action(action: NavigationAction<Self::PageMapper>) -> Self {
            Self::NavigationAction(action)
        }
    }

    #[derive(Debug, Hash, Eq, PartialEq, Clone)]
    pub enum Page {
        AticlePage,
        ListPage,
        Settings,
    }

    impl DrawerNavigatorMapper for Page {
        type Message = Message;

        fn title(&self) -> String {
            match self {
                Page::AticlePage => "Article".to_owned(),
                Page::ListPage => "List".to_owned(),
                Page::Settings => "settings".to_owned(),
            }
        }

        fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
            match self {
                Page::AticlePage => Box::new(super::article_page::Page),
                Page::ListPage => Box::new(super::list_page::Page),
                Page::Settings => Box::new(super::settings_page::Page),
            }
        }

        fn settings(&self) -> Option<DrawerSettings> {
            Some(DrawerSettings {
                mode: DrawerMode::Sliding,
                ..Default::default()
            })
        }
    }

    pub struct App {
        nav: DrawerNavigator<Message, Page>,
    }

    impl App {
        pub fn new() -> (Self, Task<Message>) {
            let (nav, task) = DrawerNavigator::new(
                [Page::AticlePage, Page::ListPage, Page::Settings],
                Page::AticlePage,
            );

            (Self { nav }, task)
        }

        pub fn update(&mut self, message: Message) -> Task<Message> {
            let Message::NavigationAction(action) = &message;

            self.nav.handle_actions(action.clone())
        }

        pub fn view(&self) -> Element<Message> {
            self.nav.view()
        }
    }
}

#[cfg(feature = "drawer")]
fn main() -> iced::Result {
    iced::application("Sliding drawer example", app::App::update, app::App::view)
        .theme(|_| iced::Theme::KanagawaLotus)
        .run_with(app::App::new)
}

#[cfg(not(feature = "drawer"))]
fn main() {
    println!("run this example with the \"drawer\" feature enabled");
}
