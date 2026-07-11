use std::hash::Hash;
use std::iter;
use std::mem::Discriminant;
use std::ops::Div;

use iced::advanced::graphics::core::window;
use iced::advanced::overlay;
use iced::advanced::renderer::Quad;
use iced::advanced::widget::Operation;
use iced::widget::Id;
use iced::{Color, Padding, Vector, touch};
use iced::{
    Element, Event, Length, Rectangle, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
        widget::{Tree, tree},
    },
};
use indexmap::IndexMap;

use crate::animation::Frame;
use crate::base::{NavigatorElement, NavigatorElementSource, NavigatorState};

type HeaderBuilder<'a, Key, Message, Theme, Renderer> =
    dyn Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a;

type DrawerBuilder<'a, Key, Message, Theme, Renderer> =
    dyn for<'b> Fn(PageParams<Key>, &'b Vec<Key>) -> Element<'a, Message, Theme, Renderer> + 'a;

type OnNavigationEnd<'a, Key, Message> = dyn Fn(Option<Key>, Key) -> Message + 'a;

#[derive(Debug, Clone)]
pub struct State<Key: Eq + Hash> {
    pub(crate) history: Vec<Key>,
    pub(crate) previous_page: Option<Key>,
    pub(crate) pending_update: bool,
    pub(crate) transition: Option<Transition>,
    pub(crate) frame: Option<Frame>,
    pub(crate) expanded: bool,
    pub(crate) overlay_pressed: bool,
    pub(crate) navigated: bool,
}

impl<Key: 'static + Eq + Hash + Clone> NavigatorState for State<Key> {
    type Key = Key;

    fn request_update(&mut self) {
        self.pending_update = true;
    }

    fn get_previous_key(&self) -> Option<&Key> {
        if self.previous_page.is_some() {
            return self.previous_page.as_ref();
        }

        self.history.get(self.history.len() - 2)
    }

    fn navigate(&mut self, page: Key) {
        self.history.push(page);
        self.navigated = true;
        self.previous_page = None;

        if self.expanded {
            self.close_drawer();
        }
    }

    fn go_back(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.previous_page = self.history.pop();
        self.navigated = true;

        if self.expanded {
            self.close_drawer();
        }
    }

    fn clear_history(&mut self) {
        if let Some(item) = self.history.pop() {
            self.history.clear();
            self.history.push(item);
        }
    }

    fn pop_history(&mut self) {
        let page_number = self.history.len();

        if page_number > 1 {
            self.history.remove(page_number - 2);
        }
    }
}

impl<Key: 'static + Eq + Hash + Clone> State<Key> {
    pub fn open_drawer(&mut self) {
        if self.expanded {
            return;
        }

        self.expanded = true;
        self.frame = Some(Frame::new().duration(0.2));
        self.transition = Some(Transition::Expandion);
    }

    pub fn close_drawer(&mut self) {
        if !self.expanded {
            return;
        }

        self.expanded = false;
        self.frame = Some(Frame::new().duration(0.2));
        self.transition = Some(Transition::Collapse);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PageParams<Key> {
    pub current_page: Key,
    pub can_go_back: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum DrawerMode {
    Fixed,
    Sliding,
}

#[derive(Debug, Clone)]
pub(crate) enum Transition {
    Expandion,
    Collapse,
}

pub struct DrawerNavigator<'a, Key, Message, Renderer = iced::Renderer>
where
    Key: Eq + Hash + Clone,
{
    id: Option<Id>,
    width: Length,
    height: Length,
    home_page: Key,
    pages: Vec<Key>,
    header_builder: Option<Box<HeaderBuilder<'a, Key, Message, Theme, Renderer>>>,
    drawer_builder: Option<Box<DrawerBuilder<'a, Key, Message, Theme, Renderer>>>,
    header_cache: NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    drawer_cache: NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    children: IndexMap<
        Discriminant<Key>,
        NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    >,
    on_navigation_end: Option<Box<OnNavigationEnd<'a, Key, Message>>>,
    mode: DrawerMode,
    overlay: bool,
}

impl<'a, Key, Message, Renderer> DrawerNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone,
{
    pub fn new(home_page: Key) -> Self {
        Self {
            id: None,
            width: Length::Fill,
            height: Length::Fill,
            header_builder: None,
            drawer_builder: None,
            drawer_cache: NavigatorElement::empty(),
            header_cache: NavigatorElement::empty(),
            pages: Vec::new(),
            children: IndexMap::new(),
            on_navigation_end: None,
            mode: DrawerMode::Sliding,
            overlay: false,
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
            .insert(disc, NavigatorElementSource::Direct(page.into()).into());
        self.pages.push(key);

        self
    }

    pub fn insert_page_with(
        mut self,
        key: Key,
        fun: impl Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        let disc = std::mem::discriminant(&key);
        let item = NavigatorElementSource::Closure(Box::new(fun));

        self.children.insert(disc, item.into());
        self.pages.push(key);

        self
    }

    pub fn drawer_widget(
        mut self,
        fun: impl Fn(PageParams<Key>, &Vec<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.drawer_builder = Some(Box::new(fun));

        self
    }

    pub fn header_widget(
        mut self,
        fun: impl Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.header_builder = Some(Box::new(fun));

        self
    }

    pub fn on_navigation_end(
        mut self,
        on_navigation_end: impl Fn(Option<Key>, Key) -> Message + 'a,
    ) -> Self {
        self.on_navigation_end = Some(Box::new(on_navigation_end));

        self
    }

    pub fn mode(mut self, mode: DrawerMode) -> Self {
        self.mode = mode;

        self
    }

    pub fn overlay(mut self, overlay: bool) -> Self {
        self.overlay = overlay;

        self
    }
}

impl<'a, Key, Message, Renderer> Widget<Message, Theme, Renderer>
    for DrawerNavigator<'a, Key, Message, Renderer>
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
            frame: None,
            transition: None,
            history: vec![self.home_page.clone()],
            expanded: false,
            overlay_pressed: false,
            navigated: false,
        })
    }

    fn children(&self) -> Vec<Tree> {
        let mut children: Vec<_> = self.children.iter().map(|(_, _)| Tree::empty()).collect();

        children.push(Tree::empty());
        children.push(Tree::empty());

        children
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Key>>();

        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);

        let page_index = self.children.get_index_of(&disc).unwrap();
        let header_index = children_len - 1;
        let drawer_index = children_len - 2;

        let (header_layout, drawer_layout, page_layout) = get_layout(
            layout,
            self.mode,
            !self.header_cache.is_empty(),
            !self.drawer_cache.is_empty(),
        );

        if let Some(header) = self.header_cache.get_element_mut() {
            operation.traverse(&mut |operation| {
                header.as_widget_mut().operate(
                    &mut tree.children[header_index],
                    header_layout.unwrap(),
                    renderer,
                    operation,
                );
            });
        }

        if let Some(drawer) = self.drawer_cache.get_element_mut() {
            operation.traverse(&mut |operation| {
                drawer.as_widget_mut().operate(
                    &mut tree.children[drawer_index],
                    drawer_layout.unwrap(),
                    renderer,
                    operation,
                );
            });
        }

        if let Some(page) = self.children.get_mut(&disc) {
            let element = page.get_element_mut().unwrap();

            operation.traverse(&mut |operation| {
                element.as_widget_mut().operate(
                    &mut tree.children[page_index],
                    page_layout.unwrap(),
                    renderer,
                    operation,
                );
            });
        }

        operation.custom(self.id.as_ref(), layout.bounds(), state);

        if let DrawerMode::Fixed = self.mode
            && state.transition.is_some()
        {
            state.transition = None;
            state.frame = None;
        }
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > self.children.len() + 2
            && let Some(item) = tree.children.pop()
        {
            tree.children.truncate(self.children.len());
            tree.children.push(item);
        }

        while tree.children.len() < self.children.len() + 2 {
            tree.children.insert(tree.children.len() - 3, Tree::empty());
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        if self.header_builder.is_none() {
            self.header_cache.clear_cache();
        }

        if self.drawer_builder.is_none() {
            self.drawer_cache.clear_cache();
        }

        let state = tree.state.downcast_mut::<State<Key>>();

        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);
        let child_index = self.children.get_index_of(&disc).unwrap();
        let children_len = tree.children.len();

        let header_index = children_len - 1;
        let drawer_index = children_len - 2;
        let page_index = children_len - 3;

        let params = PageParams {
            current_page: key.clone(),
            can_go_back: state.history.len() > 1,
        };

        self.children
            .get_mut(&disc)
            .map(|page| {
                let mut page_header = self.header_builder.as_ref().map(|builder| {
                    let element = builder(params.clone());

                    tree.children[header_index].diff(&element);

                    element
                });

                let header_node = page_header.as_mut().map(|header| {
                    let node = header.as_widget_mut().layout(
                        &mut tree.children[header_index],
                        renderer,
                        &limits,
                    );

                    node
                });

                let page_drawer = self.drawer_builder.as_ref().map(|builder| {
                    let element = builder(params.clone(), &self.pages);

                    tree.children[drawer_index].diff(&element);

                    element
                });

                if page.is_empty() {
                    page.update_cache(params);
                }

                let page_element = page.take_element().unwrap();

                tree.children[child_index].diff(&page_element);

                let mut items = Vec::with_capacity(3);

                items.extend(page_header);
                items.extend(page_drawer);
                items.push(page_element);

                tree.children.swap(child_index, page_index);
                tree.children.swap(drawer_index, page_index);

                let node = match self.mode {
                    DrawerMode::Fixed => {
                        let mut base = if self.drawer_builder.is_some() {
                            layout::flex::resolve(
                                layout::flex::Axis::Horizontal,
                                renderer,
                                &limits,
                                self.width,
                                self.height,
                                Padding::ZERO,
                                0.0,
                                iced::Alignment::Start,
                                &mut items[if self.header_builder.is_some() { 1 } else { 0 }..],
                                &mut tree.children[page_index..=drawer_index],
                            )
                        } else {
                            items.last_mut().unwrap().as_widget_mut().layout(
                                &mut tree.children[drawer_index],
                                renderer,
                                &limits,
                            )
                        };

                        match header_node {
                            Some(header_node) => {
                                let base_size = base.size();
                                base.translate_mut(Vector::new(0.0, header_node.size().height));

                                layout::Node::with_children(base_size, [header_node, base].to_vec())
                            }
                            None => base,
                        }
                    }
                    DrawerMode::Sliding => {
                        let items_len = items.len();

                        let drawer_node = if self.drawer_builder.is_some() {
                            Some(items[items_len - 2].as_widget_mut().layout(
                                &mut tree.children[page_index],
                                renderer,
                                &limits,
                            ))
                        } else {
                            None
                        };

                        let mut page_node = items[items_len - 1].as_widget_mut().layout(
                            &mut tree.children[drawer_index],
                            renderer,
                            &limits,
                        );

                        match header_node {
                            Some(node) => {
                                let base_size = page_node.size();
                                page_node.translate_mut(Vector::new(0.0, node.size().height));

                                layout::Node::with_children(
                                    base_size,
                                    iter::once(node)
                                        .chain(drawer_node)
                                        .chain(iter::once(page_node))
                                        .collect(),
                                )
                            }
                            None => layout::Node::with_children(
                                page_node.size(),
                                iter::empty()
                                    .chain(drawer_node)
                                    .chain(iter::once(page_node))
                                    .collect(),
                            ),
                        }
                    }
                };

                if self.header_builder.is_some() {
                    self.header_cache.return_element(items.remove(0));
                }

                if self.drawer_builder.is_some() {
                    self.drawer_cache.return_element(items.remove(0));
                }

                page.return_element(items.remove(0));

                tree.children.swap(drawer_index, page_index);
                tree.children.swap(child_index, page_index);

                node
            })
            .unwrap()
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

            if state.navigated {
                state.navigated = false;

                if let Some(on_navigation_end) = self.on_navigation_end.as_ref() {
                    shell.publish(on_navigation_end(
                        state.get_previous_key().cloned(),
                        state.history.last().cloned().unwrap(),
                    ));
                }
            }

            if let Some(frame) = state.frame.as_mut() {
                if frame.is_complete() {
                    state.frame = None;
                    state.transition = None;
                } else {
                    frame.update()
                }

                shell.request_redraw();
            }
        }

        if state.transition.is_some() {
            return;
        }

        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);

        let page_index = self.children.get_index_of(&disc).unwrap();
        let header_index = children_len - 1;
        let drawer_index = children_len - 2;

        let has_drawer = self.drawer_builder.is_some()
            && (state.expanded || matches!(self.mode, DrawerMode::Fixed));

        let (header_layout, drawer_layout, page_layout) =
            get_layout(layout, self.mode, !self.header_cache.is_empty(), has_drawer);

        if let Some(child) = self.drawer_cache.get_element_mut()
            && has_drawer
        {
            child.as_widget_mut().update(
                &mut tree.children[drawer_index],
                event,
                drawer_layout.unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if let Some(header) = self.header_cache.get_element_mut()
            && (!state.expanded
                || !matches!(self.mode, DrawerMode::Sliding)
                || matches!(event, Event::Window(_)))
        {
            header.as_widget_mut().update(
                &mut tree.children[header_index],
                event,
                header_layout.unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        };

        if let DrawerMode::Sliding = self.mode
            && state.expanded
        {
            let page_bounds = page_layout.unwrap().bounds();
            let bounds = header_layout
                .map(|header| page_bounds.union(&header.bounds()))
                .unwrap_or(page_bounds);

            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if cursor.is_over(bounds) && !cursor.is_over(drawer_layout.unwrap().bounds()) {
                        state.overlay_pressed = true;
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. }) => {
                    if state.overlay_pressed {
                        state.overlay_pressed = false;

                        if cursor.is_over(bounds)
                            && !cursor.is_over(drawer_layout.unwrap().bounds())
                        {
                            state.close_drawer();
                            state.pending_update = false;
                            shell.request_redraw();
                        }
                    }
                }
                _ => {}
            };

            return;
        }

        if has_drawer
            && !matches!(event, Event::Window(_))
            && cursor.is_over(drawer_layout.unwrap().bounds())
        {
            return;
        }

        if let Some(page) = self.children.get_mut(&disc) {
            let element = page.get_element_mut().unwrap();

            element.as_widget_mut().update(
                &mut tree.children[page_index],
                event,
                page_layout.unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
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
        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);

        let page_index = self.children.get_index_of(&disc).unwrap();
        let header_index = children_len - 1;
        let drawer_index = children_len - 2;

        let has_drawer = self.drawer_builder.is_some()
            && (state.expanded || matches!(self.mode, DrawerMode::Fixed));

        let (header_layout, drawer_layout, page_layout) = get_layout(
            layout,
            self.mode,
            !self.header_cache.is_empty(),
            !self.drawer_cache.is_empty(),
        );

        let drawer_interaction = if let Some(child) = self.drawer_cache.get_element()
            && has_drawer
        {
            let layout = drawer_layout.unwrap();
            let interaction = child.as_widget().mouse_interaction(
                &tree.children[drawer_index],
                layout,
                cursor,
                viewport,
                renderer,
            );

            if cursor.is_over(layout.bounds()) {
                return interaction;
            }

            Some(interaction)
        } else {
            None
        };

        if let DrawerMode::Sliding = self.mode
            && self.overlay
            && state.expanded
        {
            return drawer_interaction.unwrap_or_default();
        }

        let header_interaction = if let Some(child) = self.header_cache.get_element().as_ref() {
            let layout = header_layout.unwrap();
            let interaction = child.as_widget().mouse_interaction(
                &tree.children[header_index],
                layout,
                cursor,
                viewport,
                renderer,
            );

            if cursor.is_over(layout.bounds()) {
                return interaction;
            }

            Some(interaction)
        } else {
            None
        };

        self.children
            .get(&disc)
            .map(|page| {
                let element = page.get_element().unwrap();

                element.as_widget().mouse_interaction(
                    &tree.children[page_index],
                    page_layout.unwrap(),
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .or(header_interaction)
            .or(drawer_interaction)
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
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let state = tree.state.downcast_ref::<State<Key>>();
            let children_len = tree.children.len();
            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);

            let page_index = self.children.get_index_of(&disc).unwrap();
            let header_index = children_len - 1;
            let drawer_index = children_len - 2;

            let (header_layout, drawer_layout, page_layout) = get_layout(
                layout,
                self.mode,
                !self.header_cache.is_empty(),
                !self.drawer_cache.is_empty(),
            );

            if let Some(child) = self.header_cache.get_element() {
                child.as_widget().draw(
                    &tree.children[header_index],
                    renderer,
                    theme,
                    style,
                    header_layout.unwrap(),
                    cursor,
                    &clipped_viewport,
                );
            }

            if let Some(page) = self.children.get(&disc) {
                let element = page.get_element().unwrap();

                element.as_widget().draw(
                    &tree.children[page_index],
                    renderer,
                    theme,
                    style,
                    page_layout.unwrap(),
                    cursor,
                    &clipped_viewport,
                );
            }

            if let Some(child) = self.drawer_cache.get_element() {
                match self.mode {
                    DrawerMode::Fixed => {
                        child.as_widget().draw(
                            &tree.children[drawer_index],
                            renderer,
                            theme,
                            style,
                            drawer_layout.unwrap(),
                            cursor,
                            &clipped_viewport,
                        );
                    }
                    DrawerMode::Sliding if state.expanded || state.transition.is_some() => {
                        let layout = drawer_layout.unwrap();

                        let overlay = if self.overlay {
                            let mut base = page_layout.unwrap().bounds();

                            if let Some(header) = header_layout {
                                base = base.union(&header.bounds());
                            }

                            Some(Quad {
                                bounds: base,
                                ..Default::default()
                            })
                        } else {
                            None
                        };

                        match state.transition.as_ref().map(|transition| {
                            transition.to_translation(state.frame.as_ref(), &layout.bounds())
                        }) {
                            Some(Some((translation, opacity))) => {
                                if self.overlay {
                                    renderer.with_layer(clipped_viewport, |renderer| {
                                        renderer.fill_quad(
                                            overlay.unwrap(),
                                            Color::from_rgba(0.0, 0.0, 0.0, opacity),
                                        );
                                    });
                                }

                                renderer.with_translation(
                                    Vector::new(translation, 0.0),
                                    |renderer| {
                                        renderer.with_layer(clipped_viewport, |renderer| {
                                            child.as_widget().draw(
                                                &tree.children[drawer_index],
                                                renderer,
                                                theme,
                                                style,
                                                layout,
                                                cursor,
                                                &clipped_viewport,
                                            );
                                        });
                                    },
                                );
                            }
                            _ => {
                                if self.overlay {
                                    renderer.with_layer(clipped_viewport, |renderer| {
                                        renderer.fill_quad(
                                            overlay.unwrap(),
                                            Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                                        );
                                    });
                                }

                                renderer.with_layer(clipped_viewport, |renderer| {
                                    child.as_widget().draw(
                                        &tree.children[drawer_index],
                                        renderer,
                                        theme,
                                        style,
                                        layout,
                                        cursor,
                                        &clipped_viewport,
                                    );
                                });
                            }
                        };
                    }
                    _ => {}
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
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let state = tree.state.downcast_ref::<State<Key>>();

            if state.pending_update || state.transition.is_some() {
                return None;
            }

            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);

            let page_index = self.children.get_index_of(&disc).unwrap();

            let (header_layout, drawer_layout, page_layout) = get_layout(
                layout,
                self.mode,
                !self.header_cache.is_empty(),
                !self.drawer_cache.is_empty(),
            );

            let (header_state, tree_slice) = tree.children.split_last_mut().unwrap();
            let (drawer_state, tree_slice) = tree_slice.split_last_mut().unwrap();
            let page_state = &mut tree_slice[page_index];

            let header_overlay = self.header_cache.get_element_mut().map(|element| {
                element.as_widget_mut().overlay(
                    header_state,
                    header_layout.unwrap(),
                    renderer,
                    &clipped_viewport,
                    translation,
                )
            });

            let drawer_overlay = self.drawer_cache.get_element_mut().map(|element| {
                if let DrawerMode::Sliding = self.mode
                    && !state.expanded
                    && state.transition.is_none()
                {
                    return None;
                }

                let drawer_layout = drawer_layout.unwrap();

                let drawer_translation = state.transition.as_ref().map(|transition| {
                    transition
                        .to_translation(state.frame.as_ref(), &drawer_layout.bounds())
                        .map(|(value, _)| value)
                        .unwrap_or(0.0)
                });

                let translation = Vector {
                    x: translation.x + drawer_translation.unwrap_or(0.0),
                    y: translation.y,
                };

                element.as_widget_mut().overlay(
                    drawer_state,
                    drawer_layout,
                    renderer,
                    viewport,
                    translation,
                )
            });

            self.children.get_mut(&disc).map(|page| {
                let element = page.get_element_mut().unwrap();

                let page_overlay = element.as_widget_mut().overlay(
                    page_state,
                    page_layout.unwrap(),
                    renderer,
                    &clipped_viewport,
                    translation,
                );

                overlay::Group::with_children(
                    drawer_overlay
                        .into_iter()
                        .chain(header_overlay)
                        .flatten()
                        .chain(page_overlay)
                        .collect(),
                )
                .overlay()
            })
        } else {
            None
        }
    }
}

impl Transition {
    fn to_translation(&self, frame: Option<&Frame>, area: &Rectangle) -> Option<(f32, f32)> {
        let width = area.width;
        let frame = frame?;

        match self {
            Self::Expandion => Some((
                (frame.get_value().div(100.0) - 1.0) * width,
                frame.get_value().div(100.0) * 0.6,
            )),
            Self::Collapse => Some((
                ((frame.get_value().div(100.0) - 1.0) * width).abs() - width,
                ((frame.get_value().div(100.0) * 0.6) - 0.6).abs(),
            )),
        }
    }
}

fn get_layout<'a>(
    layout: Layout<'a>,
    mode: DrawerMode,
    has_header: bool,
    has_drawer: bool,
) -> (Option<Layout<'a>>, Option<Layout<'a>>, Option<Layout<'a>>) {
    match mode {
        DrawerMode::Fixed if has_header && has_drawer => (
            Some(layout.child(0)),
            Some(layout.child(1).child(0)),
            Some(layout.child(1).child(1)),
        ),
        DrawerMode::Fixed if has_header && has_drawer => {
            (Some(layout.child(0)), None, Some(layout.child(1)))
        }
        DrawerMode::Fixed if has_header && has_drawer => {
            (None, Some(layout.child(0)), Some(layout.child(1)))
        }
        DrawerMode::Fixed => (None, None, Some(layout)),
        DrawerMode::Sliding => {
            if has_header && has_drawer {
                (
                    Some(layout.child(0)),
                    Some(layout.child(1)),
                    Some(layout.child(2)),
                )
            } else if has_header {
                (Some(layout.child(0)), None, Some(layout.child(1)))
            } else if has_drawer {
                (None, Some(layout.child(0)), Some(layout.child(1)))
            } else {
                (None, None, Some(layout.child(0)))
            }
        }
    }
}

impl<'a, Key, Message, Renderer> From<DrawerNavigator<'a, Key, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Key: 'static + Eq + Hash + Clone,
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(drawer: DrawerNavigator<'a, Key, Message, Renderer>) -> Self {
        Self::new(drawer)
    }
}

pub fn drawer_navigator<'a, Key, Message, Renderer>(
    home_page: Key,
) -> DrawerNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone,
    Renderer: iced::advanced::Renderer,
{
    DrawerNavigator::new(home_page)
}
