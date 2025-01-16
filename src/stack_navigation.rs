use iced::widget::column;

use crate::{components::header::Header, NavigationConvertible, PageComponent};

pub(crate) struct StackFrame<M>
where
    M: Clone + NavigationConvertible,
{
    header: Header<M>,
    widget: Box<dyn PageComponent<M>>,
}

impl<M> StackFrame<M>
where
    M: Clone + NavigationConvertible,
{
    pub fn new(widget: Box<dyn PageComponent<M>>) -> Self {
        Self {
            header: Header::new(widget.title().unwrap_or_else(String::new)),
            widget,
        }
    }

    pub fn show_left_button(&mut self, flag: bool) {
        self.header.show_left_button = flag;
    }

    fn view_header(&self) -> iced::Element<M> {
        self.header.view()
    }
}

impl<M> PageComponent<M> for StackFrame<M>
where
    M: Clone + NavigationConvertible,
{
    fn update(&mut self, message: M) -> iced::Task<M> {
        self.widget.update(message)
    }

    fn view(&self) -> iced::Element<M> {
        column![self.view_header(), self.widget.view()].into()
    }

    fn on_navigation(&mut self, can_go_back: bool) {
        self.header.show_left_button = can_go_back;
    }
}
