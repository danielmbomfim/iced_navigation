#![allow(mismatched_lifetime_syntaxes)]
use iced::{
    Alignment, Element, Length, Task, Theme, color,
    widget::{Row, Space, button, column, container, row, scrollable, text, text_input},
};
use iced_font_awesome::fa_icon_solid;
use iced_navigation::{
    operations::{clear_history, go_back, navigate},
    stack_navigator::{PageParams, stack_navigator},
};

#[derive(Debug, Clone)]
enum Message {
    Username(String),
    Password(String),
    Navigate(Page),
    NavigationEnded(Option<Page>, Page),
    GoBack,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Page {
    LoginPage,
    HomePage(String),
    Details(u32),
}

struct App {
    username: String,
    password: String,
    error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        App {
            password: String::new(),
            username: String::new(),
            error: None,
        }
    }
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Username(name) => {
                self.username = name;
                Task::none()
            }
            Message::Password(pass) => {
                self.password = pass;
                Task::none()
            }
            Message::Navigate(Page::HomePage(username)) => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error = Some("provide a valid username and password".to_owned());

                    return Task::none();
                }

                self.error = None;

                return navigate(Page::HomePage(username));
            }
            Message::Navigate(page) => navigate(page),
            Message::GoBack => go_back::<Message, Page>(),
            Message::NavigationEnded(previous, _current) => {
                if let Some(Page::LoginPage) = previous {
                    return clear_history::<Message, Page>();
                }

                return Task::none();
            }
        }
    }

    fn view(&self) -> Element<Message> {
        stack_navigator(Page::LoginPage)
            .header_widget(header)
            .insert_page(
                Page::LoginPage,
                login_home(&self.username, &self.password, self.error.as_ref()),
            )
            .insert_page_with(Page::HomePage(String::new()), home_page)
            .insert_page_with(Page::Details(0), details_page)
            .on_navigation_end(Message::NavigationEnded)
            .into()
    }
}

fn login_home<'a>(
    username: &'a str,
    password: &'a str,
    error: Option<&'a String>,
) -> Element<'a, Message> {
    container(
        column![
            text_input("Username", username)
                .on_input(Message::Username)
                .on_submit(Message::Navigate(Page::HomePage(username.to_owned()))),
            text_input("Password", password)
                .on_input(Message::Password)
                .on_submit(Message::Navigate(Page::HomePage(username.to_owned()))),
            button(
                container(text("Login"))
                    .align_x(Alignment::Center)
                    .width(Length::Fixed(100.0))
            )
            .on_press(Message::Navigate(Page::HomePage(username.to_owned()))),
        ]
        .push(error.map(|message| text(message).color(color!(255, 0, 0))))
        .align_x(Alignment::Center)
        .spacing(10)
        .max_width(400.0),
    )
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn home_page<'a>(params: PageParams<Page>) -> Element<'a, Message> {
    let name = match params.page {
        Page::HomePage(ref username) => Some(username),
        _ => None,
    };

    column![scrollable(
        column![
            container(text!("Welcome {}!", name.as_ref().unwrap()).size(30))
                .align_x(Alignment::Center),
            text(concat!(
                "Sed ut perspiciatis unde omnis iste natus error sit voluptatem ",
                "accusantium doloremque laudantium, totam rem aperiam, eaque ipsa ",
                "quae ab illo inventore veritatis et quasi architecto beatae vitae ",
                "dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit ",
                "aspernatur aut odit aut fugit, sed quia consequuntur magni dolores ",
                "eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, ",
                "qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, ",
                "sed quia non numquam eius modi tempora incidunt ut labore et dolore ",
                "magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis ",
                "nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut ",
                "aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit ",
                "qui in ea voluptate velit esse quam nihil molestiae consequatur, vel ",
                "illum qui dolorem eum fugiat quo voluptas nulla pariatur?"
            )),
            row![
                Space::new().width(Length::Fill),
                button(text("details 1")).on_press(Message::Navigate(Page::Details(1)))
            ]
            .spacing(20),
            row![
                Space::new().width(Length::Fill),
                button(text("details 2")).on_press(Message::Navigate(Page::Details(2)))
            ]
            .spacing(20)
        ]
        .padding(20)
        .spacing(10),
    )]
    .into()
}

fn details_page<'a>(params: PageParams<Page>) -> Element<'a, Message> {
    let id = match params.page {
        Page::Details(ref value) => Some(value),
        _ => None,
    };

    column![scrollable(
        column![
            container(text!("Details number {}", id.unwrap()).size(30)).align_x(Alignment::Center),
            text(concat!(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer gravida, ",
                "purus sed posuere interdum, erat nisl tincidunt tellus, quis interdum nisi ",
                "est vel nunc. Proin tristique, massa quis vestibulum posuere, nulla arcu ",
                "dignissim tellus, vel ornare nulla mauris id leo. Proin in tellus et nibh ",
                "lacinia feugiat. Maecenas hendrerit, sapien quis sodales porttitor, turpis ",
                "arcu iaculis enim, sed fringilla eros ligula eget urna. Etiam quis tincidunt ",
                "augue. Aenean lobortis nec urna sit amet dignissim. Pellentesque porta ",
                "est sit amet accumsan porta."
            )),
            text(concat!(
                "Proin suscipit, urna vitae consequat porttitor, augue metus accumsan ligula, ",
                "elementum pulvinar mauris ligula ac magna. Pellentesque vehicula, felis ",
                "id varius cursus, tellus felis hendrerit odio, vitae luctus urna quam sed ",
                "est. Vestibulum pellentesque justo finibus, sodales mi sit amet, ",
                "dapibus arcu. Aenean tempus sapien in nisi imperdiet, in posuere eros ",
                "ultrices. Curabitur id libero a magna feugiat pretium sit amet at ",
                "ex. Proin sed velit at erat eleifend pellentesque condimentum id ",
                "dolor. Nunc placerat hendrerit turpis id fringilla. Integer at ",
                "turpis varius, porttitor magna vel, fermentum est. Curabitur nec ",
                "laoreet ligula. Morbi justo mauris, malesuada eu bibendum et, ",
                "convallis ut nisl. Interdum et malesuada fames ac ante ipsum primis ",
                "in faucibus. Nunc molestie urna eget porttitor finibus. Praesent ",
                "tempus porta lacus sit amet gravida."
            )),
            text(concat!(
                "Sed ut perspiciatis unde omnis iste natus error sit voluptatem ",
                "accusantium doloremque laudantium, totam rem aperiam, eaque ipsa ",
                "quae ab illo inventore veritatis et quasi architecto beatae vitae ",
                "dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit ",
                "aspernatur aut odit aut fugit, sed quia consequuntur magni dolores ",
                "eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, ",
                "qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, ",
                "sed quia non numquam eius modi tempora incidunt ut labore et dolore ",
                "magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis ",
                "nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut ",
                "aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit ",
                "qui in ea voluptate velit esse quam nihil molestiae consequatur, vel ",
                "illum qui dolorem eum fugiat quo voluptas nulla pariatur?"
            )),
        ]
        .padding(20)
        .spacing(10),
    )]
    .into()
}

fn header<'a>(params: PageParams<Page>) -> Element<'a, Message> {
    let title = match params.page {
        Page::HomePage(_) => "Home page".to_owned(),
        Page::Details(id) => format!("Details number {id}"),
        Page::LoginPage => return None::<Option<Element<_>>>.into(),
    };

    container(
        Row::new()
            .push(if params.can_go_back {
                Some(
                    button(
                        fa_icon_solid("angle-left")
                            .style(|theme: &iced::Theme| {
                                let pallete = theme.extended_palette();
                                text::Style {
                                    color: Some(pallete.primary.base.text),
                                }
                            })
                            .size(20.0),
                    )
                    .on_press(Message::GoBack)
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
                )
            } else {
                None
            })
            .push(text(title).size(18).style(|theme: &Theme| {
                let pallete = theme.extended_palette();

                text::Style {
                    color: Some(pallete.primary.base.text),
                }
            }))
            .push(Space::new().width(Length::Fill))
            .spacing(20)
            .padding(10)
            .width(Length::Fill)
            .height(50)
            .align_y(Alignment::Center),
    )
    .style(|theme: &Theme| container::background(theme.palette().primary))
    .into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view).run()
}
