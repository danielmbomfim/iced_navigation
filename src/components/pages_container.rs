use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Div;

use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{layout, overlay, renderer};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::event::{self, Event};
use iced::mouse;
use iced::widget::container::{self, draw_background};
use iced::Theme;
use iced::{Element, Length, Rectangle, Size, Vector};

pub struct PagesContainer<'a, Message, Renderer = iced::Renderer> {
    width: Length,
    height: Length,
    disabed: HashSet<usize>,
    hidden: HashSet<usize>,
    animation_progress: Vec<Option<f32>>,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    elevated_frame: RefCell<Option<usize>>,
}

impl<'a, Message, Renderer> PagesContainer<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            children: Vec::new(),
            disabed: HashSet::new(),
            hidden: HashSet::new(),
            animation_progress: Vec::new(),
            elevated_frame: RefCell::new(None),
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn hide(mut self, index: usize, hide: bool) -> Self {
        if !hide {
            self.hidden.remove(&index);
        } else {
            self.hidden.insert(index);
        }

        self
    }

    pub fn hide_last(self, hide: bool) -> Self {
        let index = self.children.len() - 1;
        self.hide(index, hide)
    }

    pub fn disable(mut self, index: usize, disable: bool) -> Self {
        if !disable {
            self.disabed.remove(&index);
        } else {
            self.disabed.insert(index);
        }

        self
    }

    pub fn disable_last(self, disable: bool) -> Self {
        let index = self.children.len() - 1;
        self.disable(index, disable)
    }

    pub fn progress(mut self, index: usize, progress: Option<f32>) -> Self {
        self.animation_progress[index] = progress.map(|value| (value.div(100.0) - 1.0).abs());

        self
    }

    pub fn progress_last(self, progress: Option<f32>) -> Self {
        let index = self.children.len() - 1;
        self.progress(index, progress)
    }

    pub fn n_progress(mut self, index: usize, progress: Option<f32>) -> Self {
        self.animation_progress[index] = progress.map(|value| value.div(100.0));

        self
    }

    pub fn n_progress_last(self, progress: Option<f32>) -> Self {
        let index = self.children.len() - 1;
        self.n_progress(index, progress)
    }

    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();

        if self.children.is_empty() {
            let child_size = child.as_widget().size_hint();

            self.width = self.width.enclose(child_size.width);
            self.height = self.height.enclose(child_size.height);
        }

        self.children.push(child);
        self.animation_progress.push(None);
        self
    }

    pub fn extend(
        self,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Self::push)
    }

    pub fn elevate(self, index: usize) -> Self {
        {
            let mut elevated_frame = self.elevated_frame.borrow_mut();
            *elevated_frame = Some(index);
        }

        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer>
    for PagesContainer<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let mut elevated_frame = self.elevated_frame.borrow_mut();

        match elevated_frame.take() {
            Some(value) => {
                let element = tree.children.remove(value);
                tree.children.push(element);
                tree.diff_children(&self.children);
            }
            None => tree.diff_children(&self.children),
        };
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        if self.children.is_empty() {
            return layout::Node::new(limits.resolve(self.width, self.height, Size::ZERO));
        }

        let base = self.children[0]
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits);

        let size = limits.resolve(self.width, self.height, base.size());
        let limits = layout::Limits::new(Size::ZERO, size);

        let nodes =
            std::iter::once(base)
                .chain(self.children[1..].iter().zip(&mut tree.children[1..]).map(
                    |(layer, tree)| {
                        let node = layer.as_widget().layout(tree, renderer, &limits);

                        node
                    },
                ))
                .collect();

        layout::Node::with_children(size, nodes)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        mut cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let is_over_scroll = matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. }))
            && cursor.is_over(layout.bounds());

        self.children
            .iter_mut()
            .enumerate()
            .rev()
            .zip(tree.children.iter_mut().rev())
            .zip(layout.children().rev())
            .filter_map(|((item, state), layout)| {
                let (index, child) = item;

                if self.disabed.contains(&index) {
                    return None;
                }

                let status = child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );

                if is_over_scroll && cursor != mouse::Cursor::Unavailable {
                    let interaction = child
                        .as_widget()
                        .mouse_interaction(state, layout, cursor, viewport, renderer);

                    if interaction != mouse::Interaction::None {
                        cursor = mouse::Cursor::Unavailable;
                    }
                }

                Some(status)
            })
            .find(|&status| status == event::Status::Captured)
            .unwrap_or(event::Status::Ignored)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .enumerate()
            .rev()
            .zip(tree.children.iter().rev())
            .zip(layout.children().rev())
            .filter_map(|((item, state), layout)| {
                let (index, child) = item;

                if self.disabed.contains(&index) {
                    return None;
                }

                Some(
                    child
                        .as_widget()
                        .mouse_interaction(state, layout, cursor, viewport, renderer),
                )
            })
            .find(|&interaction| interaction != mouse::Interaction::None)
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let container_width = bounds.width;

        let background = theme.palette().background;
        let background_style = container::Style::default().background(background);

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            let mut layers = self
                .children
                .iter()
                .enumerate()
                .map(|(index, item)| (item, self.animation_progress[index]))
                .zip(&tree.children)
                .zip(layout.children())
                .enumerate();

            let layers = layers.by_ref();

            let mut draw_layer = |i,
                                  anim,
                                  layer: &Element<'a, Message, Theme, Renderer>,
                                  state,
                                  layout,
                                  cursor| {
                if i > 0 {
                    renderer.with_layer(clipped_viewport, |renderer| {
                        match anim {
                            Some(value) => renderer.with_translation(
                                Vector::new(container_width * value, 0.0),
                                |renderer| {
                                    draw_background(renderer, &background_style, clipped_viewport);

                                    layer.as_widget().draw(
                                        state,
                                        renderer,
                                        theme,
                                        style,
                                        layout,
                                        cursor,
                                        &clipped_viewport,
                                    )
                                },
                            ),
                            None => {
                                draw_background(renderer, &background_style, clipped_viewport);

                                layer.as_widget().draw(
                                    state,
                                    renderer,
                                    theme,
                                    style,
                                    layout,
                                    cursor,
                                    &clipped_viewport,
                                );
                            }
                        };
                    });
                } else {
                    match anim {
                        Some(value) => renderer.with_translation(
                            Vector::new(container_width * value, 0.0),
                            |renderer| {
                                draw_background(renderer, &background_style, clipped_viewport);

                                layer.as_widget().draw(
                                    state,
                                    renderer,
                                    theme,
                                    style,
                                    layout,
                                    cursor,
                                    &clipped_viewport,
                                )
                            },
                        ),
                        None => {
                            draw_background(renderer, &background_style, clipped_viewport);

                            layer.as_widget().draw(
                                state,
                                renderer,
                                theme,
                                style,
                                layout,
                                cursor,
                                &clipped_viewport,
                            );
                        }
                    };
                }
            };

            let pages_number = self.children.len();

            for (i, (((layer, animation_value), state), layout)) in layers {
                if self.hidden.contains(&i) || pages_number > 2 && i <= pages_number - 3 {
                    continue;
                }

                draw_layer(i, animation_value, layer, state, layout, cursor);
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, translation)
    }
}

impl<'a, Message, Renderer> From<PagesContainer<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(stack: PagesContainer<'a, Message, Renderer>) -> Self {
        Self::new(stack)
    }
}

pub fn pages_container<'a, Message, Renderer>() -> PagesContainer<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    PagesContainer::new()
}
