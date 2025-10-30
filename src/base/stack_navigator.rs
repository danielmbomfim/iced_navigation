use std::collections::HashMap;
use std::hash::Hash;
use std::mem::Discriminant;
use std::ops::{Div, Neg};

use iced::advanced::graphics::core::window;
use iced::advanced::widget::Operation;
use iced::widget::Id;
use iced::{
    Element, Event, Length, Rectangle, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
        widget::{Tree, tree},
    },
    widget::container::{self, draw_background},
};

use crate::animation::Frame;
use crate::base::{NavigatorPage, NavigatorState};

#[derive(Debug, Clone)]
pub struct State<Key: Eq + Hash> {
    pub(crate) previous_page: Option<Key>,
    pub(crate) history: Vec<Key>,
    pub(crate) transition: Option<Transition>,
    pub(crate) frame: Option<Frame>,
    pub(crate) pending_update: bool,
}

impl<Key: 'static + Eq + Hash + Clone> NavigatorState for State<Key> {
    type Key = Key;

    fn request_update(&mut self) {
        self.pending_update = true;
    }

    fn history_len(&self) -> usize {
        if self.previous_page.is_some() {
            self.history.len() + 1
        } else {
            self.history.len()
        }
    }

    fn get_previous_key(&self) -> Option<&Key> {
        if self.previous_page.is_some() {
            return self.previous_page.as_ref();
        }

        self.history.get(self.history.len() - 2)
    }

    fn navigate(&mut self, page: Key) {
        self.history.push(page);
        self.frame = Some(Frame::new());
        self.transition = Some(Transition::Foward);
    }

    fn go_back(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.previous_page = Some(self.history.remove(self.history.len() - 1));

        self.frame = Some(Frame::new());
        self.transition = Some(Transition::Back);
    }

    fn clear_history(&mut self) {
        if let Some(item) = self.history.pop() {
            self.history.clear();
            self.history.push(item);
        }

        self.previous_page = None;
    }

    fn pop_history(&mut self) {
        let page_number = self.history.len();

        if page_number > 1 {
            self.history.remove(page_number - 2);
            self.previous_page = None;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageParams<Key> {
    pub page: Key,
    pub can_go_back: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum Transition {
    Foward,
    Back,
}

pub struct StackNavigator<'a, Key, Message, Renderer = iced::Renderer>
where
    Key: Eq + Hash + Clone,
{
    id: Option<Id>,
    width: Length,
    height: Length,
    home_page: Key,
    cache: [Option<Element<'a, Message, Theme, Renderer>>; 2],
    children:
        HashMap<Discriminant<Key>, NavigatorPage<'a, PageParams<Key>, Message, Theme, Renderer>>,
}

impl<'a, Key, Message, Renderer> StackNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone,
{
    pub fn new(home_page: Key) -> Self {
        Self {
            id: None,
            width: Length::Fill,
            height: Length::Fill,
            children: HashMap::new(),
            cache: [None, None],
            home_page,
        }
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn insert_page(
        mut self,
        key: Key,
        page: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let disc = std::mem::discriminant(&key);

        self.children
            .insert(disc, NavigatorPage::Direct(page.into()));
        self
    }

    pub fn insert_page_with(
        mut self,
        key: Key,
        fun: impl Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'static,
    ) -> Self {
        let disc = std::mem::discriminant(&key);
        let item = NavigatorPage::Closure(Box::new(fun));

        self.children.insert(disc, item);

        self
    }
}

impl<'a, Key, Message, Renderer> Widget<Message, Theme, Renderer>
    for StackNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone + 'static,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Key>>()
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            pending_update: false,
            previous_page: None,
            history: vec![self.home_page.clone()],
            transition: None,
            frame: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::empty()]
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Key>>();

        operation.custom(self.id.as_ref(), layout.bounds(), state);

        if state.history_len() < tree.children.len() {
            tree.children.truncate(state.history_len());
        } else if state.previous_page.is_some() {
            let size = tree.children.len();
            tree.children.swap(size - 1, size - 2);
        }
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let mut size = None;
        let state = tree.state.downcast_mut::<State<Key>>();

        if state.history_len() > tree.children.len() {
            tree.children.push(Tree::empty());
        }

        let base = if let Some(transition) = state.transition.as_ref()
            && let Some(key) = state.get_previous_key()
        {
            let tree_index = state.history_len() - 2;
            let disc = std::mem::discriminant(key);

            match self.children.get_mut(&disc) {
                Some(NavigatorPage::Closure(builder)) => {
                    let params = PageParams {
                        page: key.clone(),
                        can_go_back: match transition {
                            Transition::Foward => state.history.len() > 2,
                            Transition::Back => true,
                        },
                    };

                    let mut el = builder(params);
                    tree.children[tree_index].diff(&el);

                    let node = el.as_widget_mut().layout(
                        &mut tree.children[tree_index],
                        renderer,
                        &limits,
                    );

                    self.cache[0] = Some(el);
                    Some(node)
                }
                Some(NavigatorPage::Direct(el)) => {
                    self.cache[0] = None;
                    tree.children[tree_index].diff(&*el);

                    Some(
                        el.as_widget_mut()
                            .layout(&mut tree.children[0], renderer, &limits),
                    )
                }
                _ => Some(layout::Node::new(Size::ZERO)),
            }
        } else {
            self.cache[0] = None;
            None
        };

        let head = {
            let limits = if let Some(value) = &base {
                size = Some(limits.resolve(self.width, self.height, value.size()));
                layout::Limits::new(Size::ZERO, size.unwrap())
            } else {
                limits
            };

            let tree_index = tree.children.len() - 1;
            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);

            match self.children.get_mut(&disc) {
                Some(NavigatorPage::Closure(builder)) => {
                    let params = PageParams {
                        page: key.clone(),
                        can_go_back: state.history.len() > 1,
                    };

                    let mut el = builder(params);
                    tree.children[tree_index].diff(&el);

                    let node = el.as_widget_mut().layout(
                        &mut tree.children[tree_index],
                        renderer,
                        &limits,
                    );

                    self.cache[1] = Some(el);
                    node
                }
                Some(NavigatorPage::Direct(el)) => {
                    self.cache[1] = None;
                    tree.children[tree_index].diff(&*el);

                    el.as_widget_mut()
                        .layout(&mut tree.children[tree_index], renderer, &limits)
                }
                _ => layout::Node::new(Size::ZERO),
            }
        };

        let mut nodes = Vec::with_capacity(2);

        if let Some(node) = base {
            nodes.push(node);
        } else {
            size = Some(limits.resolve(self.width, self.height, head.size()));
        }

        nodes.push(head);
        layout::Node::with_children(size.unwrap(), nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Key>>();

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            if state.pending_update {
                state.pending_update = false;
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }

            if let Some(frame) = state.frame.as_mut() {
                if frame.is_complete() {
                    state.frame = None;
                    state.transition = None;

                    if state.previous_page.is_some() {
                        state.previous_page = None;
                        tree.children.remove(tree.children.len() - 2);
                        shell.invalidate_layout();
                        shell.request_redraw();
                        return;
                    }
                } else {
                    frame.update()
                }

                shell.request_redraw();
            }
        }

        if state.transition.is_some() {
            return;
        }

        if let Some(value) = self.cache.last_mut() {
            let widget_state = tree.children.last_mut().unwrap();
            let mut children_layout: Vec<_> = layout.children().collect();
            let layout = children_layout.last_mut().unwrap();

            match value {
                Some(element) => {
                    element.as_widget_mut().update(
                        widget_state,
                        event,
                        *layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                None => {
                    let key = state.history.last().unwrap();
                    let disc = std::mem::discriminant(key);

                    let widget = self.children.get_mut(&disc).unwrap();

                    if let NavigatorPage::Direct(element) = widget {
                        element.as_widget_mut().update(
                            widget_state,
                            event,
                            *layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                }
            }
        };
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Key>>();

        if state.transition.is_some() {
            return mouse::Interaction::default();
        }

        if let Some(value) = self.cache.last() {
            let widget_state = tree.children.last().unwrap();
            let mut children_layout: Vec<_> = layout.children().collect();
            let layout = children_layout.last_mut().unwrap();

            match value {
                Some(element) => element.as_widget().mouse_interaction(
                    widget_state,
                    *layout,
                    cursor,
                    viewport,
                    renderer,
                ),
                None => {
                    let key = state.history.last().unwrap();
                    let disc = std::mem::discriminant(key);

                    let widget = self.children.get(&disc).unwrap();

                    if let NavigatorPage::Direct(element) = widget {
                        element.as_widget().mouse_interaction(
                            widget_state,
                            *layout,
                            cursor,
                            viewport,
                            renderer,
                        )
                    } else {
                        mouse::Interaction::default()
                    }
                }
            }
        } else {
            mouse::Interaction::default()
        }
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
        if self.children.is_empty() {
            return;
        }

        let bounds = layout.bounds();
        let nav_state = tree.state.downcast_ref::<State<Key>>();

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            let children_layout: Vec<_> = layout.children().collect();
            let children_len = tree.children.len();

            let (main_transition, base_transition) = nav_state
                .transition
                .as_ref()
                .map(|transition| transition.into_translation(nav_state.frame.as_ref(), &bounds))
                .unwrap_or((None, None));

            if nav_state.transition.is_some()
                && let Some(value) = self.cache.get(0)
                && let Some(key) = nav_state.get_previous_key()
            {
                let history_state = tree.children.get(children_len - 2).unwrap();
                let history_layout = children_layout[0];

                match value {
                    Some(element) => draw_layer(
                        base_transition,
                        element,
                        &history_state,
                        renderer,
                        theme,
                        style,
                        history_layout,
                        cursor,
                        &clipped_viewport,
                    ),
                    None => {
                        let disc = std::mem::discriminant(key);

                        let widget = self.children.get(&disc).unwrap();

                        if let NavigatorPage::Direct(element) = widget {
                            draw_layer(
                                base_transition,
                                element,
                                &history_state,
                                renderer,
                                theme,
                                style,
                                history_layout,
                                cursor,
                                &clipped_viewport,
                            );
                        }
                    }
                };
            }

            if let Some(value) = self.cache.last() {
                let head_state = tree.children.get(children_len - 1).unwrap();
                let head_layout = children_layout.last().unwrap();

                match value {
                    Some(element) => renderer.with_layer(clipped_viewport, |renderer| {
                        draw_layer(
                            main_transition,
                            element,
                            head_state,
                            renderer,
                            theme,
                            style,
                            *head_layout,
                            cursor,
                            &clipped_viewport,
                        );
                    }),
                    None => {
                        let key = nav_state.history.last().unwrap();
                        let disc = std::mem::discriminant(key);

                        let widget = self.children.get(&disc).unwrap();

                        if let NavigatorPage::Direct(element) = widget {
                            renderer.with_layer(clipped_viewport, |renderer| {
                                draw_layer(
                                    main_transition,
                                    element,
                                    head_state,
                                    renderer,
                                    theme,
                                    style,
                                    *head_layout,
                                    cursor,
                                    &clipped_viewport,
                                );
                            });
                        }
                    }
                }
            }
        }
    }
}

fn draw_layer<'a, Message, Renderer>(
    translation: Option<f32>,
    layer: &Element<'a, Message, Theme, Renderer>,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let background = theme.palette().background;
    let background_style = container::Style::default().background(background);

    match translation {
        Some(value) => renderer.with_translation(Vector::new(value, 0.0), |renderer| {
            draw_background(renderer, &background_style, *viewport);
            layer
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, &viewport)
        }),
        None => {
            draw_background(renderer, &background_style, *viewport);
            layer
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, &viewport);
        }
    };
}

impl Transition {
    fn into_translation(
        &self,
        frame: Option<&Frame>,
        area: &Rectangle,
    ) -> (Option<f32>, Option<f32>) {
        let width = area.width;
        let frame = match frame {
            Some(value) => value,
            None => return (None, None),
        };

        match self {
            Self::Foward => {
                let main = (frame.get_value().div(100.0) - 1.0).abs() * width;
                let base = ((frame.get_value() * 0.4).div(100.0) * width).neg();

                (Some(main), Some(base))
            }
            Self::Back => {
                let main = (frame.get_value().div(100.0) * width) - width;
                let base = (frame.get_value().div(100.0) * 0.6) * width;

                (Some(main), Some(base))
            }
        }
    }
}

impl<'a, Key, Message, Renderer> From<StackNavigator<'a, Key, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Key: 'static + Eq + Hash + Clone,
    Message: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(stack: StackNavigator<'a, Key, Message, Renderer>) -> Self {
        Self::new(stack)
    }
}

pub fn stack_navigator<'a, Key, Message, Renderer>(
    home_page: Key,
) -> StackNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone,
    Renderer: iced::advanced::Renderer,
{
    StackNavigator::new(home_page)
}
