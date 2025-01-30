use iced::{Element, Task};
use iced_navigation::{
    NavigationAction, NavigationConvertible, Navigator, PageComponent, StackNavigator,
    StackNavigatorMapper,
};

#[derive(Debug, Clone)]
enum Message {
    NavigationAction(NavigationAction<Page>),
    Username(String),
    Password(String),
    LoginRequest,
}

impl NavigationConvertible for Message {
    type PageMapper = Page;

    fn from_action(action: NavigationAction<Self::PageMapper>) -> Self {
        Self::NavigationAction(action)
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Page {
    LoginPage,
    HomePage(String),
    Details(u32),
}

impl StackNavigatorMapper for Page {
    type Message = Message;

    fn title(&self) -> String {
        match self {
            Page::HomePage(_) => "Home page".to_owned(),
            Page::LoginPage => "Login page".to_owned(),
            Page::Details(id) => format!("Detaills {}", id),
        }
    }

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
        match self {
            Page::HomePage(name) => Box::new(home::HomePage::new(name.to_owned())),
            Page::LoginPage => Box::new(login::LoginPage::new()),
            Page::Details(id) => Box::new(details::DetailsPage::new(*id)),
        }
    }
}

pub mod login {
    use iced::{
        color,
        widget::{button, column, container, text, text_input},
        Alignment, Element, Length, Task,
    };
    use iced_navigation::{NavigationAction, NavigationConvertible, PageComponent};

    use crate::{Message, Page};

    pub struct LoginPage {
        username: String,
        password: String,
        error: Option<String>,
    }

    impl LoginPage {
        pub fn new() -> Self {
            Self {
                username: String::new(),
                password: String::new(),
                error: None,
            }
        }
    }

    impl PageComponent<Message> for LoginPage {
        fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::Username(name) => self.username = name,
                Message::Password(pass) => self.password = pass,
                Message::LoginRequest => {
                    if self.username.is_empty() || self.password.is_empty() {
                        self.error = Some("provide a valid username and password".to_owned());

                        return Task::none();
                    }

                    self.error = None;

                    return Task::done(Message::from_action(NavigationAction::Navigate(
                        Page::HomePage(self.username.clone()),
                    )));
                }
                _ => {}
            };

            Task::none()
        }

        fn view(&self) -> Element<Message> {
            container(
                column![
                    text_input("Username", &self.username)
                        .on_input(Message::Username)
                        .on_submit(Message::LoginRequest),
                    text_input("Password", &self.password)
                        .on_input(Message::Password)
                        .on_submit(Message::LoginRequest),
                    button(
                        container(text("Login"))
                            .align_x(Alignment::Center)
                            .width(Length::Fixed(100.0))
                    )
                    .on_press(Message::LoginRequest)
                ]
                .push_maybe(
                    self.error
                        .as_ref()
                        .map(|message| text(message).color(color!(255, 0, 0))),
                )
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
    }
}

pub mod home {
    use iced::{
        widget::{button, column, container, horizontal_space, row, scrollable, text},
        Alignment, Element, Task,
    };
    use iced_navigation::{NavigationAction, NavigationConvertible, PageComponent};

    use crate::{Message, Page};

    pub struct HomePage {
        name: String,
    }

    impl HomePage {
        pub fn new(name: String) -> Self {
            Self { name }
        }
    }

    impl PageComponent<Message> for HomePage {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            scrollable(
                column![
                    container(text!("Wellcome {}!", self.name).size(30)).align_x(Alignment::Center),
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
                        horizontal_space(),
                        button(text("details")).on_press(Message::from_action(
                            NavigationAction::Navigate(Page::Details(1))
                        ))
                    ]
                    .spacing(20)
                ]
                .padding(20)
                .spacing(10),
            )
            .into()
        }
    }
}

pub mod details {
    use iced::{
        widget::{column, container, scrollable, text},
        Alignment, Element, Task,
    };
    use iced_navigation::PageComponent;

    use crate::Message;

    pub struct DetailsPage {
        id: u32,
    }

    impl DetailsPage {
        pub fn new(id: u32) -> Self {
            Self { id }
        }
    }

    impl PageComponent<Message> for DetailsPage {
        fn update(&mut self, _message: Message) -> Task<Message> {
            Task::none()
        }

        fn view(&self) -> Element<Message> {
            scrollable(
                column![
                    container(text!("Details number {}", self.id).size(30))
                        .align_x(Alignment::Center),
                    text(concat!(
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer gravida, ",
                        "purus sed posuere interdum, erat nisl tincidunt tellus, quis interdum nisi ",
                        "est vel nunc. Proin tristique, massa quis vestibulum posuere, nulla arcu ",
                        "dignissim tellus, vel ornare nulla mauris id leo. Proin in tellus et nibh ",
                        "lacinia feugiat. Maecenas hendrerit, sapien quis sodales porttitor, turpis ",
                        "arcu iaculis enim, sed fringilla eros ligula eget urna. Etiam quis tincidunt ",
                        "augue. Aenean lobortis nec urna sit amet dignissim. Pellentesque porta ",
                        "est sit amet accumsan porta.")),
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
                        "tempus porta lacus sit amet gravida.")),
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
            )
            .into()
        }
    }
}

struct App {
    nav: StackNavigator<Message, Page>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                nav: StackNavigator::new(Page::LoginPage),
            },
            Task::none(),
        )
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::NavigationAction(action) = &message {
            if self.nav.is_on_page_and(Page::LoginPage, || {
                if let NavigationAction::Navigate(page) = action {
                    return matches![page, Page::HomePage(_)];
                }

                false
            }) {
                self.nav.clear_history();
            }

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
