# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
