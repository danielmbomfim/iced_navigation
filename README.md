# iced\_navigation

`iced_navigation` is a navigation library for the [`iced`](https://github.com/iced-rs/iced) GUI framework, designed to provide structured navigation patterns for building interactive applications.

## Features

- **Stack Navigator**: Implements a stack-based navigation system.
- **Page Mapping**: Define navigation pages and their corresponding components easily.
- **Navigation Actions**: Supports pushing, popping, and replacing pages dynamically.

## Planned Features

- **Drawer Navigator**: Side menu navigation support (coming soon).
- **Tabs Navigator**: Tab-based navigation support (coming soon).

## Installation

To use `iced_navigation`, add it to your `Cargo.toml`:

```toml
[dependencies]
iced = "0.13.1"
iced_navigation = "1.0.0"
```

## Usage

### Example: Stack Navigator

```rust
use iced::{Element, Task};
use iced_navigation::{
    stack_navigator::{StackNavigator, StackNavigatorMapper},
    NavigationAction, NavigationConvertible, PageComponent,
};

// Defines the message enum used by the application
#[derive(Debug, Clone)]
enum Message {
    NavigationAction(NavigationAction<Page>),
    // Your aplication messages are defined here
}

impl NavigationConvertible for Message {
    type PageMapper = Page;

    // Maps navigation actions to your message enum
    fn from_action(action: NavigationAction<Self::PageMapper>) -> Self {
        Self::NavigationAction(action)
    }
}

// This enum defines the pages available to the navigator
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Page {
    LoginPage,
    HomePage(String),
}

// This implementation maps pages to their titles and UI components
impl StackNavigatorMapper for Page {
    type Message = Message;

    fn title(&self) -> String {
        match self {
            Page::HomePage(_) => "Home page".to_owned(),
            Page::LoginPage => "Login page".to_owned(),
        }
    }

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
        // The page components must implement the PageComponent trait
        match self {
            Page::HomePage(name) => Box::new(HomeComponent::new(name.to_owned())),
            Page::LoginPage => Box::new(LoginComponent::new()),
        }
    }
}

struct App {
    nav: StackNavigator<Message, Page>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (nav, task) = StackNavigator::new(Page::LoginPage);

        (Self { nav }, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        // this ensures any navigation action is handled corretly by the navigator
        if let Message::NavigationAction(action) = &message {
            return self.nav.handle_actions(action.clone());
        }

        // the navigator will pass any message to the update function of the current page
        self.nav.update(message)
    }

    fn view(&self) -> Element<Message> {
        self.nav.view()
    }
}

fn main() -> iced::Result {
    iced::application("Example", App::update, App::view).run_with(App::new)
}
```

Complete examples can be found in the `examples` folder.

## Contributing

Contributions are welcome! Feel free to open issues and pull requests.

## License

This project is licensed under the MIT License.



