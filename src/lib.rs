mod widgets;

#[cfg(feature = "drawer")]
pub use widgets::drawer_navigator;
pub use widgets::operations;
#[cfg(feature = "stack")]
pub use widgets::stack_navigator;
#[cfg(feature = "tabs")]
pub use widgets::tabs_navigator;

pub(crate) mod animation;
