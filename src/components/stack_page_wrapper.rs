use std::ops::Div;

use iced::{
    advanced::{
        layout, renderer,
        widget::{tree, Operation, Tree},
        Clipboard, Layout, Renderer, Shell, Widget,
    },
    alignment, event, mouse, overlay,
    widget::container::{draw_background, layout, Style},
    Background, Color, Element, Event, Length, Padding, Rectangle, Size, Theme, Vector,
};

pub struct StackPageWrapper<'a, M, R>
where
    R: Renderer,
{
    content: Element<'a, M, Theme, R>,
    size: Size<Length>,
    progress: f32,
    active: bool,
    reversed: bool,
    animated: bool,
}

impl<'a, M, R> StackPageWrapper<'a, M, R>
where
    R: Renderer,
{
    pub fn new(content: impl Into<Element<'a, M, Theme, R>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            active: true,
            reversed: false,
            animated: false,
            progress: 0.0,
            content,
            size,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        if !self.animated {
            return self;
        }

        self.progress = if self.reversed {
            (progress - 100.0).abs()
        } else {
            progress
        }
        .div(100.0);

        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }
}

impl<'a, M, R> Widget<M, Theme, R> for StackPageWrapper<'a, M, R>
where
    R: Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.size
    }

    fn layout(&self, tree: &mut Tree, renderer: &R, limits: &layout::Limits) -> layout::Node {
        layout(
            limits,
            self.size.width.fluid(),
            self.size.height.fluid(),
            f32::INFINITY,
            f32::INFINITY,
            Padding::ZERO.left(self.progress * limits.max().width),
            alignment::Horizontal::Left,
            alignment::Vertical::Top,
            |limits| self.content.as_widget().layout(tree, renderer, limits),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) -> event::Status {
        if !self.active {
            return event::Status::Ignored;
        }

        self.content.as_widget_mut().on_event(
            tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &R,
    ) -> mouse::Interaction {
        if !self.active {
            return mouse::Interaction::None;
        }

        self.content.as_widget().mouse_interaction(
            tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut R,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let mut background_bounds = bounds.clone();
        let inverted_progress = (self.progress - 1.0).abs();

        background_bounds.width = bounds.width * inverted_progress;
        background_bounds.x = bounds.width - background_bounds.width;

        let background = theme.palette().background;
        let mut style = Style::default().background(background);

        if background.a != 1.0 {
            style = style.background(Background::Color(if theme.extended_palette().is_dark {
                Color::BLACK
            } else {
                Color::WHITE
            }));
        }

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            draw_background(renderer, &style, background_bounds);

            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: style.text_color.unwrap_or(renderer_style.text_color),
                },
                layout.children().next().unwrap(),
                cursor,
                &clipped_viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, R>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, M, R> From<StackPageWrapper<'a, M, R>> for Element<'a, M, Theme, R>
where
    M: 'a,
    R: Renderer + 'a,
{
    fn from(column: StackPageWrapper<'a, M, R>) -> Element<'a, M, Theme, R> {
        Element::new(column)
    }
}

pub fn stack_page_wrapper<'a, M, R>(
    content: impl Into<Element<'a, M, Theme, R>>,
) -> StackPageWrapper<'a, M, R>
where
    R: Renderer,
{
    StackPageWrapper::new(content)
}
