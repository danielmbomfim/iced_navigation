use std::collections::HashMap;
use std::hash::Hash;
use std::mem::Discriminant;
use std::ops::{Div, Neg};

use iced::Padding;
use iced::advanced::graphics::core::window;
use iced::advanced::overlay;
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
        self.previous_page = None;
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
    header_cache: [Option<Element<'a, Message, Theme, Renderer>>; 2],
    children:
        HashMap<Discriminant<Key>, NavigatorPage<'a, PageParams<Key>, Message, Theme, Renderer>>,
    header_element: Option<NavigatorPage<'a, PageParams<Key>, Message, Theme, Renderer>>,
    on_navigation_end: Option<Box<dyn Fn(Option<Key>, Key) -> Message + 'a>>,
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
            header_cache: [None, None],
            home_page,
            header_element: None,
            on_navigation_end: None,
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
        fun: impl Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        let disc = std::mem::discriminant(&key);
        let item = NavigatorPage::Closure(Box::new(fun));

        self.children.insert(disc, item);

        self
    }

    pub fn header_widget(
        mut self,
        fun: impl Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        let item = NavigatorPage::Closure(Box::new(fun));

        self.header_element = Some(item);

        self
    }

    pub fn on_navigation_end(
        mut self,
        on_navigation_end: impl Fn(Option<Key>, Key) -> Message + 'a,
    ) -> Self {
        self.on_navigation_end = Some(Box::new(on_navigation_end));

        self
    }
}

impl<'a, Key, Message, Renderer> Widget<Message, Theme, Renderer>
    for StackNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone + 'static,
    Message: Clone,
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
        vec![Tree::empty(), Tree::empty()]
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

        if state.history_len() + 2 < tree.children.len() {
            tree.children.truncate(state.history_len() + 2);
            return;
        } else if state.previous_page.is_some() {
            let size = tree.children.len();

            tree.children.swap(size - 1, size - 3);
            tree.children.swap(size - 2, size - 4);
            return;
        }

        if state.history_len() + 2 > tree.children.len() {
            if tree.children.len() == 2 {
                tree.children.push(Tree::empty());
                tree.children.push(Tree::empty());
            } else {
                tree.children.push(Tree::empty());

                let size = tree.children.len();

                tree.children.swap(size - 2, size - 3);
                tree.children.swap(size - 4, size - 5);
            }
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

        let base = if let Some(transition) = state.transition.as_ref()
            && let Some(key) = state.get_previous_key()
        {
            let page_index = tree.children.len() - 3;
            let header_index = tree.children.len() - 4;

            let disc = std::mem::discriminant(key);
            let mut items: Vec<_> = Vec::with_capacity(2);

            if let Some(NavigatorPage::Closure(builder)) = self.header_element.as_ref() {
                let params = PageParams {
                    page: key.clone(),
                    can_go_back: match transition {
                        Transition::Foward => state.history.len() > 2,
                        Transition::Back => true,
                    },
                };

                let el = builder(params);

                tree.children[header_index].diff(&el);

                items.push(el);
            } else {
                self.header_cache[0] = None;
            };

            let (page, moved) =
                if let Some(NavigatorPage::Closure(builder)) = self.children.get_mut(&disc) {
                    let params = PageParams {
                        page: key.clone(),
                        can_go_back: match transition {
                            Transition::Foward => state.history.len() > 2,
                            Transition::Back => true,
                        },
                    };

                    let el = builder(params);
                    tree.children[page_index].diff(&el);

                    (Some(el), false)
                } else if let Some(NavigatorPage::Direct(el)) =
                    self.children.insert(disc, NavigatorPage::None)
                {
                    tree.children[page_index].diff(&el);

                    (Some(el), true)
                } else {
                    (None, false)
                };

            if let Some(page) = page {
                items.push(page);
            }

            let node = layout::flex::resolve(
                layout::flex::Axis::Vertical,
                renderer,
                &limits,
                self.width,
                self.height,
                Padding::ZERO,
                0.0,
                iced::Alignment::Start,
                &mut items,
                &mut tree.children[if self.header_element.is_some() {
                    header_index
                } else {
                    page_index
                }..page_index + 1],
            );

            if items.len() > 1 {
                self.header_cache[0] = Some(items.remove(0));
            } else {
                self.header_cache[0] = None;
            }

            if moved {
                self.children
                    .insert(disc, NavigatorPage::Direct(items.remove(0)));
                self.cache[0] = None;
            } else {
                self.cache[0] = Some(items.remove(0));
            }

            Some(node)
        } else {
            self.cache[0] = None;
            None
        };

        let main_page = {
            let limits = if let Some(value) = &base {
                size = Some(limits.resolve(self.width, self.height, value.size()));
                layout::Limits::new(Size::ZERO, size.unwrap())
            } else {
                limits
            };

            let page_index = tree.children.len() - 1;
            let header_index = tree.children.len() - 2;

            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);
            let mut items: Vec<_> = Vec::with_capacity(2);

            if let Some(NavigatorPage::Closure(builder)) = self.header_element.as_ref() {
                let params = PageParams {
                    page: key.clone(),
                    can_go_back: state.history.len() > 1,
                };

                let el = builder(params);

                tree.children[header_index].diff(&el);

                items.push(el);
            } else {
                self.header_cache[1] = None;
            };

            let (page, moved) =
                if let Some(NavigatorPage::Closure(builder)) = self.children.get_mut(&disc) {
                    let params = PageParams {
                        page: key.clone(),
                        can_go_back: state.history.len() > 1,
                    };

                    let el = builder(params);
                    tree.children[page_index].diff(&el);

                    (Some(el), false)
                } else if let Some(NavigatorPage::Direct(el)) =
                    self.children.insert(disc, NavigatorPage::None)
                {
                    tree.children[page_index].diff(&el);

                    (Some(el), true)
                } else {
                    (None, false)
                };

            if let Some(page) = page {
                items.push(page);
            }

            let node = layout::flex::resolve(
                layout::flex::Axis::Vertical,
                renderer,
                &limits,
                self.width,
                self.height,
                Padding::ZERO,
                0.0,
                iced::Alignment::Start,
                &mut items,
                &mut tree.children[if self.header_element.is_some() {
                    header_index
                } else {
                    page_index
                }..page_index + 1],
            );

            if items.len() > 1 {
                self.header_cache[1] = Some(items.remove(0));
            } else {
                self.header_cache[1] = None;
            }

            if moved {
                self.children
                    .insert(disc, NavigatorPage::Direct(items.remove(0)));
                self.cache[1] = None;
            } else {
                self.cache[1] = Some(items.remove(0));
            }

            node
        };

        let mut nodes = Vec::with_capacity(2);

        if let Some(node) = base {
            nodes.push(node);
        } else {
            size = Some(limits.resolve(self.width, self.height, main_page.size()));
        }

        nodes.push(main_page);
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

                    if let Some(previous) = state.previous_page.as_ref() {
                        tree.children.remove(tree.children.len() - 3);

                        if tree.children.len() == 3 {
                            tree.children.remove(0);
                        } else {
                            let len = tree.children.len();

                            tree.children.swap(len - 3, len - 4);
                        }

                        shell.invalidate_layout();
                        shell.request_redraw();

                        if let Some(on_navigation_end) = self.on_navigation_end.as_ref() {
                            shell.publish(on_navigation_end(
                                Some(previous.clone()),
                                state.history.last().cloned().unwrap(),
                            ));
                        }
                        return;
                    } else if let Some(on_navigation_end) = self.on_navigation_end.as_ref() {
                        shell.publish(on_navigation_end(
                            state.get_previous_key().cloned(),
                            state.history.last().cloned().unwrap(),
                        ));
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

        let layout = layout.children().last().unwrap();

        if let Some(header) = self.header_cache[1].as_mut() {
            let header_index = tree.children.len() - 2;

            header.as_widget_mut().update(
                &mut tree.children[header_index],
                event,
                layout.child(0),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if let Some(value) = self.cache.last_mut() {
            let widget_state = tree.children.last_mut().unwrap();

            match value {
                Some(element) => {
                    element.as_widget_mut().update(
                        widget_state,
                        event,
                        layout.child(if self.header_cache[1].is_some() { 1 } else { 0 }),
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
                            layout.child(if self.header_cache[1].is_some() { 1 } else { 0 }),
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

        let children_number = tree.children.len();

        let page_state = &tree.children[children_number - 1];
        let header_state = &tree.children[children_number - 2];

        let layout = layout.children().last().unwrap();

        let header_interaction = if let Some(header) = self.header_cache[1].as_ref() {
            Some(header.as_widget().mouse_interaction(
                header_state,
                layout.child(0),
                cursor,
                viewport,
                renderer,
            ))
        } else {
            None
        };

        header_interaction.unwrap_or_else(|| match self.cache[1].as_ref() {
            Some(element) => element.as_widget().mouse_interaction(
                page_state,
                layout.child(if self.header_cache[1].is_some() { 1 } else { 0 }),
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
                        page_state,
                        layout.child(if self.header_cache[1].is_some() { 1 } else { 0 }),
                        cursor,
                        viewport,
                        renderer,
                    )
                } else {
                    mouse::Interaction::default()
                }
            }
        })
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

        if let Some(mut clipped_viewport) = bounds.intersection(viewport) {
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
                let mut clipped_viewport = clipped_viewport.clone();
                let page_state = tree.children.get(children_len - 3).unwrap();
                let page_layout = children_layout[0].children().collect::<Vec<_>>();

                match value {
                    Some(element) => {
                        if let Some(element) = self.header_cache[0].as_ref() {
                            let header_state = tree.children.get(children_len - 4).unwrap();

                            let offset = page_layout[0].bounds().height;

                            draw_layer(
                                base_transition,
                                element,
                                header_state,
                                renderer,
                                theme,
                                style,
                                page_layout[0],
                                cursor,
                                &clipped_viewport,
                            );

                            clipped_viewport.height -= offset;
                            clipped_viewport.y += offset;
                        }

                        draw_layer(
                            base_transition,
                            element,
                            &page_state,
                            renderer,
                            theme,
                            style,
                            *page_layout.last().unwrap(),
                            cursor,
                            &clipped_viewport,
                        );
                    }
                    None => {
                        let disc = std::mem::discriminant(key);

                        let widget = self.children.get(&disc).unwrap();

                        if let NavigatorPage::Direct(element) = widget {
                            if let Some(element) = self.header_cache[0].as_ref() {
                                let header_state = tree.children.get(children_len - 4).unwrap();

                                let offset = page_layout[0].bounds().height;

                                draw_layer(
                                    base_transition,
                                    element,
                                    header_state,
                                    renderer,
                                    theme,
                                    style,
                                    page_layout[0],
                                    cursor,
                                    &clipped_viewport,
                                );

                                clipped_viewport.height -= offset;
                                clipped_viewport.y += offset;
                            }

                            draw_layer(
                                base_transition,
                                element,
                                &page_state,
                                renderer,
                                theme,
                                style,
                                *page_layout.last().unwrap(),
                                cursor,
                                &clipped_viewport,
                            );
                        }
                    }
                };
            }

            let page_state = tree.children.get(children_len - 1).unwrap();
            let page_layout = children_layout
                .last()
                .unwrap()
                .children()
                .collect::<Vec<_>>();

            match self.cache[1].as_ref() {
                Some(element) => renderer.with_layer(clipped_viewport, |renderer| {
                    if let Some(element) = self.header_cache[1].as_ref() {
                        let header_state = tree.children.get(children_len - 2).unwrap();

                        let offset = page_layout[0].bounds().height;

                        draw_layer(
                            main_transition,
                            element,
                            header_state,
                            renderer,
                            theme,
                            style,
                            page_layout[0],
                            cursor,
                            &clipped_viewport,
                        );

                        clipped_viewport.height -= offset;
                        clipped_viewport.y += offset;
                    }

                    draw_layer(
                        main_transition,
                        element,
                        page_state,
                        renderer,
                        theme,
                        style,
                        *page_layout.last().unwrap(),
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
                            if let Some(element) = self.header_cache[1].as_ref() {
                                let header_state = tree.children.get(children_len - 2).unwrap();
                                let offset = page_layout[0].bounds().height;

                                draw_layer(
                                    main_transition,
                                    element,
                                    header_state,
                                    renderer,
                                    theme,
                                    style,
                                    page_layout[0],
                                    cursor,
                                    &clipped_viewport,
                                );

                                clipped_viewport.height -= offset;
                                clipped_viewport.y += offset;
                            }

                            draw_layer(
                                main_transition,
                                element,
                                page_state,
                                renderer,
                                theme,
                                style,
                                *page_layout.last().unwrap(),
                                cursor,
                                &clipped_viewport,
                            );
                        });
                    }
                }
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let bounds = layout.bounds();

        let nav_state: &State<Key> = tree.state.downcast_ref();
        let (main_transition, _base_transition) = nav_state
            .transition
            .as_ref()
            .map(|transition| transition.into_translation(nav_state.frame.as_ref(), &bounds))
            .unwrap_or((None, None));

        if let Some(mut clipped_viewport) = bounds.intersection(viewport) {
            if nav_state.pending_update {
                return None;
            }

            let translation = Vector {
                x: translation.x + main_transition.unwrap_or(0.0),
                y: translation.y,
            };

            let children_layout: Vec<_> = layout.children().collect();

            let page_layout = children_layout
                .last()
                .unwrap()
                .children()
                .collect::<Vec<_>>();

            match self.cache[1].as_mut() {
                Some(element) => {
                    let (page_state, tree_slice) = tree.children.split_last_mut().unwrap();

                    let header_overlay = self.header_cache[1].as_mut().map(|element| {
                        let offset = page_layout[0].bounds().height;

                        let overlay = element.as_widget_mut().overlay(
                            tree_slice.last_mut().unwrap(),
                            page_layout[0],
                            renderer,
                            viewport,
                            translation,
                        );

                        clipped_viewport.height -= offset;
                        clipped_viewport.y += offset;

                        overlay
                    });

                    let page_overlay = element.as_widget_mut().overlay(
                        page_state,
                        *page_layout.last().unwrap(),
                        renderer,
                        &clipped_viewport,
                        translation,
                    );

                    return Some(
                        overlay::Group::with_children(
                            header_overlay
                                .into_iter()
                                .flatten()
                                .chain(page_overlay)
                                .collect(),
                        )
                        .overlay(),
                    );
                }
                None => {
                    let key = nav_state.history.last().unwrap();
                    let disc = std::mem::discriminant(key);

                    let widget = self.children.get_mut(&disc).unwrap();

                    if let NavigatorPage::Direct(element) = widget {
                        let (page_state, tree_slice) = tree.children.split_last_mut().unwrap();

                        let header_overlay = self.header_cache[1].as_mut().map(|element| {
                            let offset = page_layout[0].bounds().height;

                            let overlay = element.as_widget_mut().overlay(
                                tree_slice.last_mut().unwrap(),
                                page_layout[0],
                                renderer,
                                &clipped_viewport,
                                translation,
                            );

                            clipped_viewport.height -= offset;
                            clipped_viewport.y += offset;

                            overlay
                        });

                        let page_overlay = element.as_widget_mut().overlay(
                            page_state,
                            *page_layout.last().unwrap(),
                            renderer,
                            &clipped_viewport,
                            translation,
                        );

                        return Some(
                            overlay::Group::with_children(
                                header_overlay
                                    .into_iter()
                                    .flatten()
                                    .chain(page_overlay)
                                    .collect(),
                            )
                            .overlay(),
                        );
                    }
                }
            }
        }

        None
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
    Message: 'a + Clone,
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
