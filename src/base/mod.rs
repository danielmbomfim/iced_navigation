use iced::Element;

pub mod operations;
#[cfg(feature = "stack")]
pub mod stack_navigator;

pub(crate) enum NavigatorPage<'a, Params, Message, Theme, Renderer = iced::Renderer> {
    Direct(Element<'a, Message, Theme, Renderer>),
    Closure(Box<dyn Fn(Params) -> Element<'a, Message, Theme, Renderer>>),
}

pub(crate) trait NavigatorState {
    type Key;

    fn request_update(&mut self);

    fn history_len(&self) -> usize;

    fn get_previous_key(&self) -> Option<&Self::Key>;

    fn navigate(&mut self, page: Self::Key);

    fn go_back(&mut self);

    fn pop_history(&mut self);

    fn clear_history(&mut self);
}
