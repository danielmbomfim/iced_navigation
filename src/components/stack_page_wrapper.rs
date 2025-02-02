use std::ops::Div;

use iced::{
    advanced::{
        layout, renderer,
        widget::{tree, Operation, Tree},
        Clipboard, Layout, Renderer as IcedRenderer, Shell, Widget,
    },
    alignment, event, mouse, overlay,
    widget::container::{draw_background, layout, Style},
    Element, Event, Length, Padding, Rectangle, Size, Theme, Vector,
};

pub struct StackPageWrapper<'a, Message, Renderer>
where
    Renderer: IcedRenderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    progress: f32,
    active: bool,
    animated: bool,
    hiden: bool,
}

impl<'a, Message, Renderer> StackPageWrapper<'a, Message, Renderer>
where
    Renderer: IcedRenderer,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();

        Self {
            active: true,
            hiden: false,
            animated: false,
            progress: 0.0,
            content,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn hide(mut self, hide: bool) -> Self {
        self.hiden = hide;
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

        self.progress = (progress.div(100.0) - 1.0).abs();

        self
    }

    pub fn n_progress(mut self, progress: f32) -> Self {
        if !self.animated {
            return self;
        }

        self.progress = progress.div(100.0);

        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer>
    for StackPageWrapper<'a, Message, Renderer>
where
    Renderer: IcedRenderer,
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
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            limits,
            Length::Fill,
            Length::Fill,
            f32::INFINITY,
            f32::INFINITY,
            Padding::ZERO,
            alignment::Horizontal::Left,
            alignment::Vertical::Top,
            |limits| self.content.as_widget().layout(tree, renderer, limits),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
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
        renderer: &Renderer,
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
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if self.hiden {
            return;
        }

        let bounds = layout.bounds();
        let container_width = bounds.width;

        let background = theme.palette().background;
        let style = Style::default().background(background);

        let Some(visible_bounds) = bounds.intersection(viewport) else {
            return;
        };

        renderer.with_layer(visible_bounds, |renderer| {
            renderer.with_translation(
                Vector::new(container_width * self.progress, 0.0),
                |renderer| {
                    draw_background(renderer, &style, *viewport);

                    self.content.as_widget().draw(
                        tree,
                        renderer,
                        theme,
                        &renderer::Style {
                            text_color: style.text_color.unwrap_or(renderer_style.text_color),
                        },
                        layout.children().next().unwrap(),
                        cursor,
                        viewport,
                    );
                },
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<StackPageWrapper<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: IcedRenderer + 'a,
{
    fn from(
        column: StackPageWrapper<'a, Message, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(column)
    }
}

pub fn stack_page_wrapper<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> StackPageWrapper<'a, Message, Renderer>
where
    Renderer: IcedRenderer,
{
    StackPageWrapper::new(content)
}
