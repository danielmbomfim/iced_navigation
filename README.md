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
iced = "0.14"
iced_navigation = "2.0.0"
```

## Usage

### Example: Stack Navigator

```rust
use iced::{Element, Task};
use iced_navigation::{
    operations::{go_back, navigate},
    stack_navigator::stack_navigator,
};

#[derive(Debug, Clone)]
enum Message {
    Navigate(Page),
    GoBack,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Page {
    LoginPage,
    HomePage,
}

struct App;

fn login_page<'a>() -> Element<'a, Message> {
    todo!()
}

fn home_page<'a>() -> Element<'a, Message> {
    todo!()
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(page) => navigate(page),
            Message::GoBack => go_back::<Message, Page>(),
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        stack_navigator(Page::LoginPage)
            .insert_page(Page::LoginPage, login_page())
            .insert_page(Page::HomePage, home_page())
            .into()
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}
```

### Example: Tabs Navigator

To use tab navigation, you must first enable the tabs feature in your Cargo.toml file:

```toml
[dependencies]
iced = "0.14"
iced_navigation = { version = "2.0.0", features = ["tabs"] }
```

```rust
use iced::{Element, Task};
use iced_navigation::{
    operations::navigate,
    tabs_navigator::{Mode, tabs_navigator},
};

#[derive(Debug, Clone, Copy)]
enum Message {
    Navigate(Page),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Page {
    ArticlePage,
    ListPage,
    SettingsPage,
}

fn article_page<'a>() -> Element<'a, Message> {
    todo!()
}

fn list_page<'a>() -> Element<'a, Message> {
    todo!()
}

fn settings_page<'a>() -> Element<'a, Message> {
    todo!()
}

struct App;

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(page) => navigate(page),
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        tabs_navigator(Page::ArticlePage)
            .mode(Mode::Bottom)
            .insert_page(Page::ArticlePage, article_page())
            .insert_page(Page::ListPage, list_page())
            .insert_page(Page::SettingsPage, settings_page())
            .into()
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}
```

### Example: Drawer Navigator

To use drawer navigation, you must first enable the drawer feature in your Cargo.toml file:

```toml
[dependencies]
iced = "0.14"
iced_navigation = { version = "2.0.0", features = ["drawer"] }
```

```rust
use iced::{Element, Task};
use iced_navigation::{
    drawer_navigator::{DrawerMode, drawer_navigator},
    operations::{navigate, open_drawer},
};

#[derive(Debug, Clone, Copy)]
enum Message {
    Navigate(Page),
    OpenDrawer,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Page {
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

fn article_page<'a>() -> Element<'a, Message> {
    todo!()
}

fn list_page<'a>() -> Element<'a, Message> {
    todo!()
}

fn settings_page<'a>() -> Element<'a, Message> {
    todo!()
}

struct App;

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(page) => navigate(page),
            Message::OpenDrawer => open_drawer::<Message, Page>(),
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        drawer_navigator(Page::ArticlePage)
            .mode(DrawerMode::Sliding)
            .insert_page(Page::ArticlePage, article_page())
            .insert_page(Page::ListPage, list_page())
            .insert_page(Page::SettingsPage, settings_page())
            .into()
    }
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view).run()
}
```

## Examples

Complete working examples are available in the `examples` directory:

### Stack Login Example

**File**: [examples/stack_login.rs](examples/stack_login.rs)

Demonstrates a stack-based navigation flow with login functionality. Features username/password input and navigation between login, home, and details pages.

```bash
cargo run --example stack_login
```

### Bottom Tabs Example

**File**: [examples/bottom_tabs.rs](examples/bottom_tabs.rs)

Shows tab-based navigation with tabs positioned at the bottom of the window. Includes Article, List, and Settings pages with icon-based tab indicators.

```bash
cargo run --example bottom_tabs --features tabs
```

### Drawer Example

**File**: [examples/drawer.rs](examples/drawer.rs)

Demonstrates drawer-based navigation with a side menu that slides in/out. Features Article, List, and Settings pages accessible through a drawer menu.

```bash
cargo run --example drawer --features drawer
```

### Nested Navigators Example

**File**: [examples/nested_navigators.rs](examples/nested_navigators.rs)

Advanced example combining both stack and tabs navigation. Demonstrates how to nest multiple navigator types together for more complex UI hierarchies.

```bash
cargo run --example nested_navigators --features "stack tabs"
```

## Contributing

Contributions are welcome! Feel free to open issues and pull requests.

## License

This project is licensed under the MIT License.
