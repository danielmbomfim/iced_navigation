# iced\_navigation
[![Crates.io](https://img.shields.io/crates/v/iced_navigation?style=flat-square)](https://crates.io/crates/iced_navigation)
[![Crates.io](https://img.shields.io/crates/d/iced_navigation?style=flat-square)](https://crates.io/crates/iced_navigation)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)

`iced_navigation` is a navigation library for the [`iced`](https://github.com/iced-rs/iced) GUI framework, designed to provide structured navigation patterns for building interactive applications.

| Stack navigation | Tab navigation | Drawer navigation |
|---------|---------|---------|
|![stack](https://github.com/user-attachments/assets/8bc2ccf6-ef11-492e-a147-2df5ee5ce76d)|![tabs](https://github.com/user-attachments/assets/47cb7b50-520d-4b27-bde1-69ae9cc944c9)|![drawer](https://github.com/user-attachments/assets/ef0bfa69-8db3-4049-9ab8-f9807c87cdd9)|


## Features

- **Stack Navigator**: Implements a stack-based navigation system.
- **Tabs Navigator**: Implements a tab-based navigation system.
- **Drawer Navigator**: Implements a drawer-based navigation system.
- **Page Mapping**: Define navigation pages and their corresponding components easily.
- **Navigation Actions**: Supports pushing, popping, and replacing pages dynamically.

## Installation

To use `iced_navigation`, add it to your `Cargo.toml`:

```toml
[dependencies]
iced = "0.13.1"
iced_navigation = "1.5.0"
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

### Example: Tabs Navigator

To use tab navigation, you must first enable the tabs feature in your Cargo.toml file:

```toml
[dependencies]
iced = "0.13.1"
iced_navigation = { version = "1.5.0", features = ["tabs"] }
```

```rust
use iced::{Element, Task};
use iced_navigation::{
    tabs_navigator::{TabsNavigator, TabsNavigatorMapper},
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
    SettingsPage,
    HomePage,
}

// This implementation maps pages to their titles and UI components
impl TabsNavigatorMapper for Page {
    type Message = Message;

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
        // The page components must implement the PageComponent trait
        match self {
            Page::HomePage => Box::new(HomeComponent::new()),
            Page::SettingsPage => Box::new(SettingsComponent::new()),
        }
    }
}

struct App {
    nav: TabsNavigator<Message, Page>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (nav, task) = TabsNavigator::new([Page::HomePage, Page::SettingsPage], Page::HomePage);

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

### Example: Drawer Navigator

To use drawer navigation, you must first enable the drawer feature in your Cargo.toml file:

```toml
[dependencies]
iced = "0.13.1"
iced_navigation = { version = "1.5.0", features = ["drawer"] }
```

```rust
use iced::{Element, Task};
use iced_navigation::{
    drawer_navigator::{DrawerNavigator, DrawerNavigatorMapper},
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
    SettingsPage,
    HomePage,
}

// This implementation maps pages to their titles and UI components
impl DrawerNavigatorMapper for Page {
    type Message = Message;

    fn title(&self) -> String {
        match self {
            Page::HomePage => "Home page".to_owned(),
            Page::SettingsPage => "Login page".to_owned(),
        }
    }

    fn into_component(&self) -> Box<dyn PageComponent<Self::Message>> {
        // The page components must implement the PageComponent trait
        match self {
            Page::HomePage => Box::new(HomeComponent::new()),
            Page::SettingsPage => Box::new(SettingsComponent::new()),
        }
    }
}

struct App {
    nav: DrawerNavigator<Message, Page>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (nav, task) =
            DrawerNavigator::new([Page::HomePage, Page::SettingsPage], Page::HomePage);

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



