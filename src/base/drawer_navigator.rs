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
use crate::base::{NavigatorPage, NavigatorState};

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
    header_element:
        Option<Box<dyn Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a>>,
    drawer_element: Option<
        Box<
            dyn for<'b> Fn(PageParams<Key>, &'b Vec<Key>) -> Element<'a, Message, Theme, Renderer>
                + 'a,
        >,
    >,
    cache: [Option<Element<'a, Message, Theme, Renderer>>; 3],
    children:
        IndexMap<Discriminant<Key>, NavigatorPage<'a, PageParams<Key>, Message, Theme, Renderer>>,
    on_navigation_end: Option<Box<dyn Fn(Option<Key>, Key) -> Message + 'a>>,
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
            header_element: None,
            drawer_element: None,
            pages: Vec::new(),
            children: IndexMap::new(),
            cache: [None, None, None],
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
            .insert(disc, NavigatorPage::Direct(page.into()));
        self.pages.push(key);

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
        self.pages.push(key);

        self
    }

    pub fn drawer_widget(
        mut self,
        fun: impl Fn(PageParams<Key>, &Vec<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.drawer_element = Some(Box::new(fun));

        self
    }

    pub fn header_widget(
        mut self,
        fun: impl Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.header_element = Some(Box::new(fun));

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
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Key>>();

        operation.custom(self.id.as_ref(), layout.bounds(), state);

        if let DrawerMode::Fixed = self.mode
            && state.transition.is_some()
        {
            state.transition = None;
            state.frame = None;
        }
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > self.children.len() + 2 {
            if let Some(item) = tree.children.pop() {
                tree.children.truncate(self.children.len());
                tree.children.push(item);
            }
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
        self.cache = [None, None, None];

        let state = tree.state.downcast_mut::<State<Key>>();
        let mut items = Vec::with_capacity(2);

        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);
        let child_index = self.children.get_index_of(&disc).unwrap();
        let children_len = tree.children.len();

        let header_index = children_len - 1;
        let drawer_index = children_len - 2;
        let page_index = children_len - 3;

        let (page, moved) = {
            if let Some(NavigatorPage::Closure(builder)) = self.children.get(&disc) {
                let params = PageParams {
                    current_page: key.clone(),
                    can_go_back: state.history.len() > 1,
                };

                let el = builder(params);

                tree.children[child_index].diff(&el);

                (Some(el), false)
            } else if let Some(NavigatorPage::Direct(el)) =
                self.children.insert(disc, NavigatorPage::None)
            {
                tree.children[child_index].diff(&el);

                (Some(el), true)
            } else {
                (None, false)
            }
        };

        if let Some(builder) = self.header_element.as_ref() {
            let params = PageParams {
                current_page: key.clone(),
                can_go_back: state.history.len() > 1,
            };

            let el = builder(params);

            tree.children[header_index].diff(&el);

            items.push(el);
        }

        if let Some(builder) = self.drawer_element.as_ref() {
            let params = PageParams {
                current_page: key.clone(),
                can_go_back: state.history.len() > 1,
            };

            let el = builder(params, &self.pages);

            tree.children[drawer_index].diff(&el);

            items.push(el);
        }

        items.push(page.unwrap());

        tree.children.swap(child_index, page_index);
        tree.children.swap(drawer_index, page_index);

        let header_node = if self.header_element.is_some() {
            let node = items[0].as_widget_mut().layout(
                &mut tree.children[header_index],
                renderer,
                &limits,
            );

            limits.shrink(Size::new(0.0, node.size().height));
            Some(node)
        } else {
            None
        };

        let node = match self.mode {
            DrawerMode::Fixed => {
                let mut base = if self.drawer_element.is_some() {
                    layout::flex::resolve(
                        layout::flex::Axis::Horizontal,
                        renderer,
                        &limits,
                        self.width,
                        self.height,
                        Padding::ZERO,
                        0.0,
                        iced::Alignment::Start,
                        &mut items[if self.header_element.is_some() { 1 } else { 0 }..],
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

                let drawer_node = if self.drawer_element.is_some() {
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

        if self.header_element.is_some() {
            self.cache[2] = Some(items.remove(0));
        }

        if self.drawer_element.is_some() {
            self.cache[1] = Some(items.remove(0));
        }

        if moved {
            self.children
                .insert(disc, NavigatorPage::Direct(items.remove(0)));
        } else {
            self.cache[0] = Some(items.remove(0));
        }

        tree.children.swap(drawer_index, page_index);
        tree.children.swap(child_index, page_index);

        node
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

        let has_drawer = self.drawer_element.is_some()
            && (state.expanded || matches!(self.mode, DrawerMode::Fixed));

        let (header_layout, drawer_layout, page_layout) = match self.mode {
            DrawerMode::Fixed if self.header_element.is_some() && self.drawer_element.is_some() => {
                (
                    Some(layout.child(0)),
                    Some(layout.child(1).child(0)),
                    Some(layout.child(1).child(1)),
                )
            }
            DrawerMode::Fixed if self.header_element.is_some() && self.drawer_element.is_none() => {
                (Some(layout.child(0)), None, Some(layout.child(1)))
            }
            DrawerMode::Fixed if self.header_element.is_none() && self.drawer_element.is_some() => {
                (None, Some(layout.child(0)), Some(layout.child(1)))
            }
            DrawerMode::Fixed => (None, None, Some(layout)),
            DrawerMode::Sliding => {
                if self.header_element.is_some() && self.drawer_element.is_some() {
                    (
                        Some(layout.child(0)),
                        Some(layout.child(1)),
                        Some(layout.child(2)),
                    )
                } else if self.header_element.is_some() {
                    (Some(layout.child(0)), None, Some(layout.child(1)))
                } else if self.drawer_element.is_some() {
                    (None, Some(layout.child(0)), Some(layout.child(1)))
                } else {
                    (None, None, Some(layout.child(0)))
                }
            }
        };

        if let Some(child) = self.cache[1].as_mut()
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

        self.cache[2].as_mut().map(|child| {
            if state.expanded
                && matches!(self.mode, DrawerMode::Sliding)
                && !matches!(event, Event::Window(_))
            {
                return;
            }

            child.as_widget_mut().update(
                &mut tree.children[header_index],
                event,
                header_layout.unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        });

        if let DrawerMode::Sliding = self.mode
            && state.expanded
        {
            let mut bounds = page_layout.unwrap().bounds();

            if let Some(header) = header_layout {
                bounds = bounds.union(&header.bounds());
            }

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

        match self.cache[0].as_mut() {
            Some(child) => {
                child.as_widget_mut().update(
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
            None => {
                if let Some(NavigatorPage::Direct(child)) = self.children.get_mut(&disc) {
                    child.as_widget_mut().update(
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
        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);

        let page_index = self.children.get_index_of(&disc).unwrap();
        let header_index = children_len - 1;
        let drawer_index = children_len - 2;

        let has_drawer = self.drawer_element.is_some()
            && (state.expanded || matches!(self.mode, DrawerMode::Fixed));

        let (header_layout, drawer_layout, page_layout) = match self.mode {
            DrawerMode::Fixed if self.header_element.is_some() && self.drawer_element.is_some() => {
                (
                    Some(layout.child(0)),
                    Some(layout.child(1).child(0)),
                    Some(layout.child(1).child(1)),
                )
            }
            DrawerMode::Fixed if self.header_element.is_some() && self.drawer_element.is_none() => {
                (Some(layout.child(0)), None, Some(layout.child(1)))
            }
            DrawerMode::Fixed if self.header_element.is_none() && self.drawer_element.is_some() => {
                (None, Some(layout.child(0)), Some(layout.child(1)))
            }
            DrawerMode::Fixed => (None, None, Some(layout)),
            DrawerMode::Sliding => {
                if self.header_element.is_some() && self.drawer_element.is_some() {
                    (
                        Some(layout.child(0)),
                        Some(layout.child(1)),
                        Some(layout.child(2)),
                    )
                } else if self.header_element.is_some() {
                    (Some(layout.child(0)), None, Some(layout.child(1)))
                } else if self.drawer_element.is_some() {
                    (None, Some(layout.child(0)), Some(layout.child(1)))
                } else {
                    (None, None, Some(layout.child(0)))
                }
            }
        };

        let drawer_interaction = if let Some(child) = self.cache[1].as_ref()
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

        let header_interaction = if let Some(child) = self.cache[2].as_ref() {
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

        let page_interaction = match self.cache[0].as_ref() {
            Some(child) => Some(child.as_widget().mouse_interaction(
                &tree.children[page_index],
                page_layout.unwrap(),
                cursor,
                viewport,
                renderer,
            )),
            None => {
                if let Some(NavigatorPage::Direct(child)) = self.children.get(&disc) {
                    Some(child.as_widget().mouse_interaction(
                        &tree.children[page_index],
                        page_layout.unwrap(),
                        cursor,
                        viewport,
                        renderer,
                    ))
                } else {
                    None
                }
            }
        };

        page_interaction
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

            let (header_layout, drawer_layout, page_layout) = match self.mode {
                DrawerMode::Fixed
                    if self.header_element.is_some() && self.drawer_element.is_some() =>
                {
                    (
                        Some(layout.child(0)),
                        Some(layout.child(1).child(0)),
                        Some(layout.child(1).child(1)),
                    )
                }
                DrawerMode::Fixed
                    if self.header_element.is_some() && self.drawer_element.is_none() =>
                {
                    (Some(layout.child(0)), None, Some(layout.child(1)))
                }
                DrawerMode::Fixed
                    if self.header_element.is_none() && self.drawer_element.is_some() =>
                {
                    (None, Some(layout.child(0)), Some(layout.child(1)))
                }
                DrawerMode::Fixed => (None, None, Some(layout)),
                DrawerMode::Sliding => {
                    if self.header_element.is_some() && self.drawer_element.is_some() {
                        (
                            Some(layout.child(0)),
                            Some(layout.child(1)),
                            Some(layout.child(2)),
                        )
                    } else if self.header_element.is_some() {
                        (Some(layout.child(0)), None, Some(layout.child(1)))
                    } else if self.drawer_element.is_some() {
                        (None, Some(layout.child(0)), Some(layout.child(1)))
                    } else {
                        (None, None, Some(layout.child(0)))
                    }
                }
            };

            if let Some(child) = self.cache[2].as_ref() {
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

            match self.cache[0].as_ref() {
                Some(child) => {
                    child.as_widget().draw(
                        &tree.children[page_index],
                        renderer,
                        theme,
                        style,
                        page_layout.unwrap(),
                        cursor,
                        &clipped_viewport,
                    );
                }
                None => {
                    if let Some(NavigatorPage::Direct(child)) = self.children.get(&disc) {
                        child.as_widget().draw(
                            &tree.children[page_index],
                            renderer,
                            theme,
                            style,
                            page_layout.unwrap(),
                            cursor,
                            &clipped_viewport,
                        );
                    }
                }
            };

            if let Some(child) = self.cache[1].as_ref() {
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
                            transition.into_translation(state.frame.as_ref(), &layout.bounds())
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

            let (header_layout, drawer_layout, page_layout) = match self.mode {
                DrawerMode::Fixed
                    if self.header_element.is_some() && self.drawer_element.is_some() =>
                {
                    (
                        Some(layout.child(0)),
                        Some(layout.child(1).child(0)),
                        Some(layout.child(1).child(1)),
                    )
                }
                DrawerMode::Fixed
                    if self.header_element.is_some() && self.drawer_element.is_none() =>
                {
                    (Some(layout.child(0)), None, Some(layout.child(1)))
                }
                DrawerMode::Fixed
                    if self.header_element.is_none() && self.drawer_element.is_some() =>
                {
                    (None, Some(layout.child(0)), Some(layout.child(1)))
                }
                DrawerMode::Fixed => (None, None, Some(layout)),
                DrawerMode::Sliding => {
                    if self.header_element.is_some() && self.drawer_element.is_some() {
                        (
                            Some(layout.child(0)),
                            Some(layout.child(1)),
                            Some(layout.child(2)),
                        )
                    } else if self.header_element.is_some() {
                        (Some(layout.child(0)), None, Some(layout.child(1)))
                    } else if self.drawer_element.is_some() {
                        (None, Some(layout.child(0)), Some(layout.child(1)))
                    } else {
                        (None, None, Some(layout.child(0)))
                    }
                }
            };

            let [page_cache, drawer_cache, header_cache] = &mut self.cache;

            let (header_state, tree_slice) = tree.children.split_last_mut().unwrap();
            let (drawer_state, tree_slice) = tree_slice.split_last_mut().unwrap();
            let page_state = &mut tree_slice[page_index];

            let header_overlay = header_cache.as_mut().map(|element| {
                element.as_widget_mut().overlay(
                    header_state,
                    header_layout.unwrap(),
                    renderer,
                    &clipped_viewport,
                    translation,
                )
            });

            let page_overlay = if let Some(element) = page_cache.as_mut() {
                element.as_widget_mut().overlay(
                    page_state,
                    page_layout.unwrap(),
                    renderer,
                    &clipped_viewport,
                    translation,
                )
            } else {
                if let Some(NavigatorPage::Direct(element)) = self.children.get_mut(&disc) {
                    let overlay = element.as_widget_mut().overlay(
                        page_state,
                        page_layout.unwrap(),
                        renderer,
                        &clipped_viewport,
                        translation,
                    );

                    overlay
                } else {
                    None
                }
            };

            let drawer_overlay = drawer_cache.as_mut().map(|element| {
                if let DrawerMode::Sliding = self.mode
                    && !state.expanded
                    && state.transition.is_none()
                {
                    return None;
                }

                let drawer_layout = drawer_layout.unwrap();

                let drawer_translation = state.transition.as_ref().map(|transition| {
                    transition
                        .into_translation(state.frame.as_ref(), &drawer_layout.bounds())
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

            return Some(
                overlay::Group::with_children(
                    drawer_overlay
                        .into_iter()
                        .chain(header_overlay)
                        .flatten()
                        .chain(page_overlay)
                        .collect(),
                )
                .overlay(),
            );
        }

        None
    }
}

impl Transition {
    fn into_translation(&self, frame: Option<&Frame>, area: &Rectangle) -> Option<(f32, f32)> {
        let width = area.width;
        let frame = match frame {
            Some(value) => value,
            None => return None,
        };

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
