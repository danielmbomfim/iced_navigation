# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.4.1](https://github.com/danielmbomfim/iced_navigation/compare/iced_navigation-v1.4.0...iced_navigation-v1.4.1) - 2025-05-06

### Other

- update pop_history method in Navigator trait and StackNavigator to remove return type

## [1.4.0](https://github.com/danielmbomfim/iced_navigation/compare/iced_navigation-v1.3.1...iced_navigation-v1.4.0) - 2025-05-06

### Added

- implement pop_history method in Navigator trait and StackNavigator

## [1.3.1](https://github.com/danielmbomfim/iced_navigation/compare/iced_navigation-v1.3.0...iced_navigation-v1.3.1) - 2025-05-05

### Fixed

- fix page title handling

## [1.3.0](https://github.com/danielmbomfim/iced_navigation/compare/v1.2.0...v1.3.0) - 2025-05-04

### Added

- *(tabs)* add derive macros for automating trait implementations
- *(stack)* add support to custom header components when using derive
- *(stack)* add derive macros for automating trait implementations

### Other

- add bottom tabs example using derive macros
- update macro dependencies and adapt code to API changes
- *(stack)* create a stack customization example using the new macros
- *(stack)* create a stack example using the new macros

## [1.2.0](https://github.com/danielmbomfim/iced_navigation/compare/v1.1.0...v1.2.0) - 2025-03-03

### Added

- *(tabs)* implement state persistence for the pages
- replaces iced Stack with a custom widget to manage the navigator pages

### Fixed

- fix method used to determine which pages need to be rendered by the PagesContainer
- fix bug causing navigators with more than 2 pages to not render top page

## [1.1.0](https://github.com/danielmbomfim/iced_navigation/compare/v1.0.0...v1.1.0) - 2025-02-23

### Added

- update tabs settings
- update tabs styling
- add basic implementation of a tabs navigator

### Fixed

- fix example title
- *(tabs)* fix default icon colors

### Other

- *(tabs)* fix compilation error in example
- update README
- *(tabs)* create a bottom tabs example
- release v1.0.0

## [1.0.0](https://github.com/danielmbomfim/iced_navigation/releases/tag/v1.0.0) - 2025-02-03

### Added

- add on_load method to execute tasks when a page is loaded
- *(stack)* implement a option to hide header
- *(stack)* remove redundant settings API
- remove unnecessary trait bounds and simplify generics
- *(stack)* improve navigation transition
- implements navigators helper methods
- updates header default theming
- *(stack)* adds a overlay over secondary windows
- implements going back animation
- implements basic navigation animation
- implements the stack effect
- *(stack)* improves customization
- implements header back button
- implements stack navigation type
- implemts basic navigation

### Fixed

- fix typo in login example
- correct typo in module name
- fixes bug where stack pages where losing previous state
- *(stack)* fixes scroll issues when navigating between pages
- *(stack)* fixes default color used by the default back button
- *(stack)* fixes header bug when hidding back button

### Other

- update metadata fields in Cargo.toml
- bumps version to 1.0.0
- create release workflow
- add gif example to readme
- add MIT license file
- add README file
- refactor code structure for better organization
- *(stack)* remove unecessary page elements included on the stack
- *(stack)* implement a header customization example
- *(stack)* update login example to hide header on login page
- add example demonstrating stack navigator with login flow
- *(stack)* refactor navigation animation
- refactor dtack navigator animation
- refactor stack navigator to improve customization
