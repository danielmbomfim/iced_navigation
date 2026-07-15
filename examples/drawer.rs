mod app {
    use iced::{
        Alignment, Element, Length, Task, Theme,
        widget::{Column, Space, button, column, container, row, scrollable, text},
    };
    use iced_font_awesome::fa_icon_solid;
    use iced_navigation::{
        drawer_navigator::{DrawerMode, PageParams, drawer_navigator},
        operations::{navigate, open_drawer},
    };

    #[derive(Debug, Clone, Copy)]
    pub enum Message {
        Navigate(Page),
        OpenDrawer,
    }

    #[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
    pub enum Page {
        ArticlePage,
        ListPage,
        SettingsPage,
    }

    impl Page {
        fn title(&self) -> String {
            match self {
                Self::ArticlePage => "Article".to_owned(),
                Self::ListPage => "List".to_owned(),
                Self::SettingsPage => "Settings".to_owned(),
            }
        }
    }

    pub struct App;

    impl App {
        pub fn new() -> (Self, Task<Message>) {
            (Self, Task::none())
        }

        pub fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::Navigate(page) => navigate(page),
                Message::OpenDrawer => open_drawer::<Message, Page>(),
            }
        }

        pub fn view<'a>(&'a self) -> Element<'a, Message> {
            drawer_navigator(Page::ArticlePage)
                .mode(DrawerMode::Sliding)
                .header_widget(header)
                .drawer_widget(drawer)
                .overlay(true)
                .insert_page(Page::ArticlePage, article_page())
                .insert_page(Page::ListPage, list_page())
                .insert_page(Page::SettingsPage, settings_page())
                .into()
        }
    }

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

    fn article_page<'a>() -> Element<'a, Message> {
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
            ].padding(20).height(Length::Fill).width(Length::Fill).into()
    }

    fn list_page<'a>() -> Element<'a, Message> {
        container(scrollable(
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
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(20)
                .spacing(10),
        ))
        .height(Length::Fill)
        .into()
    }

    fn settings_page<'a>() -> Element<'a, Message> {
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
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(10)
        .padding(20)
        .into()
    }

    fn drawer<'a>(params: PageParams<Page>, pages: &Vec<Page>) -> Element<'a, Message> {
        container(
            pages
                .iter()
                .fold(Column::new(), |column, page| {
                    let selected = *page == params.current_page;

                    column.push(
                        button(text(page.title()).style(move |theme: &iced::Theme| {
                            let pallete = theme.extended_palette();

                            text::Style {
                                color: if selected {
                                    Some(pallete.primary.base.text)
                                } else {
                                    Some(pallete.primary.strong.text)
                                },
                            }
                        }))
                        .on_press_maybe(if selected {
                            None
                        } else {
                            Some(Message::Navigate(*page))
                        })
                        .width(Length::Fill)
                        .style(move |theme: &iced::Theme, status| match status {
                            button::Status::Hovered => button::Style {
                                background: {
                                    Some(iced::Background::Color({
                                        let mut color =
                                            theme.extended_palette().primary.strong.color;

                                        color.a = 0.6;

                                        color
                                    }))
                                },
                                ..Default::default()
                            },
                            _ => button::Style {
                                background: Some(if !selected {
                                    iced::Background::Color(theme.palette().primary)
                                } else {
                                    iced::Background::Color({
                                        let mut color =
                                            theme.extended_palette().primary.strong.color;

                                        color.a = 0.6;

                                        color
                                    })
                                }),
                                ..Default::default()
                            },
                        }),
                    )
                })
                .width(300)
                .spacing(10)
                .padding(10)
                .align_x(Alignment::Center),
        )
        .style(|theme: &Theme| container::background(theme.palette().primary))
        .height(Length::Fill)
        .into()
    }

    fn header<'a>(params: PageParams<Page>) -> Element<'a, Message> {
        let title = params.current_page.title();

        container(
            row![
                button(
                    fa_icon_solid("bars")
                        .style(|theme: &iced::Theme| {
                            let pallete = theme.extended_palette();
                            text::Style {
                                color: Some(pallete.primary.base.text),
                            }
                        })
                        .size(20.0),
                )
                .on_press(Message::OpenDrawer)
                .style(|theme: &Theme, status| match status {
                    button::Status::Active | button::Status::Pressed => button::Style {
                        background: Some(iced::Background::Color(theme.palette().primary)),
                        ..Default::default()
                    },
                    button::Status::Hovered => button::Style {
                        background: {
                            let mut color = theme.palette().primary;
                            color.a = 0.6;
                            Some(iced::Background::Color(color))
                        },
                        ..Default::default()
                    },
                    button::Status::Disabled => button::Style {
                        background: {
                            let mut color = theme.palette().primary;
                            color.a = 0.3;
                            Some(iced::Background::Color(color))
                        },
                        ..Default::default()
                    },
                })
                .width(40)
                .height(30),
                text(title).size(18).style(|theme: &Theme| {
                    let pallete = theme.extended_palette();

                    text::Style {
                        color: Some(pallete.primary.base.text),
                    }
                }),
                Space::new().width(Length::Fill)
            ]
            .spacing(20)
            .padding(10)
            .width(Length::Fill)
            .height(50)
            .align_y(Alignment::Center),
        )
        .style(|theme: &Theme| container::background(theme.palette().primary))
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(app::App::new, app::App::update, app::App::view).run()
}
