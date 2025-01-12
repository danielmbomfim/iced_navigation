use iced::widget::{button, column, text, Row};

use crate::{NavigationAction, PageComponent};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub struct Header {
    title: String,
    show_left_button: bool,
}

impl Header {
    fn new(title: String) -> Self {
        Self {
            title,
            show_left_button: true,
        }
    }
}

impl Header {
    pub fn view<'a, M, P>(&'a self) -> iced::Element<'a, M>
    where
        P: Into<Box<dyn PageComponent<M>>> + Eq + PartialEq + Hash + Debug + Copy,
        M: From<NavigationAction<M, P>> + Clone + 'a,
    {
        Row::new()
            .push_maybe(if self.show_left_button {
                Some(button(text("voltar")).on_press(NavigationAction::GoBack.into()))
            } else {
                None
            })
            .push(text(&self.title))
            .spacing(20)
            .into()
    }
}

impl<M, P> StackFrame<M, P>
where
    P: Into<Box<dyn PageComponent<M>>> + Eq + PartialEq + Hash + Debug + Copy,
    M: From<NavigationAction<M, P>> + Clone,
{
    pub fn new(widget: Box<dyn PageComponent<M>>) -> Self {
        Self {
            header: Header::new(widget.title().unwrap_or_else(String::new)),
            widget,
            _page_marker: PhantomData,
        }
    }

    pub fn show_left_button(&mut self, flag: bool) {
        self.header.show_left_button = flag;
    }
}

pub(crate) struct StackFrame<M, P>
where
    P: Into<Box<dyn PageComponent<M>>> + Eq + PartialEq + Hash + Debug + Copy,
    M: From<NavigationAction<M, P>> + Clone,
{
    header: Header,
    widget: Box<dyn PageComponent<M>>,
    _page_marker: PhantomData<P>,
}

impl<M, P> PageComponent<M> for StackFrame<M, P>
where
    P: Into<Box<dyn PageComponent<M>>> + Eq + PartialEq + Hash + Debug + Copy,
    M: From<NavigationAction<M, P>> + Clone,
{
    fn update(&mut self, message: M) -> iced::Task<M> {
        self.widget.update(message)
    }

    fn view(&self) -> iced::Element<M> {
        column![self.header.view(), self.widget.view()].into()
    }

    fn on_navigation(&mut self, can_go_back: bool) {
        self.header.show_left_button = can_go_back;
    }
}
