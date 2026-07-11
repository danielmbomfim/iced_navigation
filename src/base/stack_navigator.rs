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
use crate::base::{NavigatorElement, NavigatorElementSource, NavigatorState};

type HeaderBuilder<'a, Key, Message, Theme, Renderer> =
    dyn Fn(PageParams<Key>) -> Element<'a, Message, Theme, Renderer> + 'a;

type OnNavigationEnd<'a, Key, Message> = dyn Fn(Option<Key>, Key) -> Message + 'a;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    NavigateFoward,
    NavigateTo(usize),
    NavigateBack,
    PopHistory,
    ClearHistory,
}

#[derive(Debug, Clone)]
pub struct State<Key: Eq + Hash> {
    pub(crate) previous_page: Option<Key>,
    pub(crate) history: Vec<Key>,
    pub(crate) transition: Option<Transition>,
    pub(crate) frame: Option<Frame>,
    pub(crate) pending_update: bool,
    pub(crate) current_action: Option<Action>,
}

impl<Key: 'static + Eq + Hash + Clone> State<Key> {
    pub(crate) fn push(&mut self, page: Key) {
        self.history.push(page);
        self.frame = Some(Frame::new());
        self.transition = Some(Transition::Foward);
        self.previous_page = None;
        self.current_action = Some(Action::NavigateFoward);
    }
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
        let disc = std::mem::discriminant(&page);

        let position = self
            .history
            .iter()
            .rposition(|element| std::mem::discriminant(element) == disc);

        match position {
            Some(index) => {
                self.previous_page = Some(self.history.remove(self.history.len() - 1));
                self.history.truncate(index + 1);
                self.frame = Some(Frame::new());
                self.transition = Some(Transition::Back);
                self.current_action = Some(Action::NavigateTo(index));
            }
            None => {
                self.history.push(page);
                self.frame = Some(Frame::new());
                self.transition = Some(Transition::Foward);
                self.previous_page = None;
                self.current_action = Some(Action::NavigateFoward);
            }
        }
    }

    fn go_back(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.previous_page = Some(self.history.remove(self.history.len() - 1));

        self.frame = Some(Frame::new());
        self.transition = Some(Transition::Back);
        self.current_action = Some(Action::NavigateBack);
    }

    fn clear_history(&mut self) {
        if let Some(item) = self.history.pop() {
            self.history.clear();
            self.history.push(item);
        }

        self.previous_page = None;
        self.current_action = Some(Action::ClearHistory);
    }

    fn pop_history(&mut self) {
        let page_number = self.history.len();

        if page_number > 1 {
            self.history.remove(page_number - 2);
            self.previous_page = None;
        }
        self.current_action = Some(Action::PopHistory);
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
    children:
        HashMap<Discriminant<Key>, NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>>,
    header_builder: Option<Box<HeaderBuilder<'a, Key, Message, Theme, Renderer>>>,
    main_header: NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    secondary_header: NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    on_navigation_end: Option<Box<OnNavigationEnd<'a, Key, Message>>>,
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
            main_header: NavigatorElement::empty(),
            secondary_header: NavigatorElement::empty(),
            home_page,
            header_builder: None,
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
            .insert(disc, NavigatorElementSource::Direct(page.into()).into());
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
            current_action: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::empty(), Tree::empty()]
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Key>>();
        let children_layout: Vec<_> = layout.children().collect();

        if state.transition.is_some()
            && let Some(key) = state.get_previous_key()
        {
            let page_index = tree.children.len() - 3;
            let header_index = tree.children.len() - 4;
            let page_layout = children_layout[0].children().collect::<Vec<_>>();

            let disc = std::mem::discriminant(key);

            if let Some(header) = self.secondary_header.get_element_mut() {
                operation.traverse(&mut |operation| {
                    header.as_widget_mut().operate(
                        &mut tree.children[header_index],
                        page_layout[0],
                        renderer,
                        operation,
                    );
                });
            }

            if let Some(page) = self.children.get_mut(&disc) {
                operation.traverse(&mut |operation| {
                    page.get_element_mut().unwrap().as_widget_mut().operate(
                        &mut tree.children[page_index],
                        *page_layout.last().unwrap(),
                        renderer,
                        operation,
                    );
                });
            }
        }

        let page_index = tree.children.len() - 1;
        let header_index = tree.children.len() - 2;
        let page_layout = children_layout
            .last()
            .unwrap()
            .children()
            .collect::<Vec<_>>();

        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);

        if let Some(header) = self.main_header.get_element_mut() {
            operation.traverse(&mut |operation| {
                header.as_widget_mut().operate(
                    &mut tree.children[header_index],
                    page_layout[0],
                    renderer,
                    operation,
                );
            });
        }

        if let Some(page) = self.children.get_mut(&disc) {
            operation.traverse(&mut |operation| {
                page.get_element_mut().unwrap().as_widget_mut().operate(
                    &mut tree.children[page_index],
                    *page_layout.last().unwrap(),
                    renderer,
                    operation,
                );
            });
        }

        operation.custom(self.id.as_ref(), layout.bounds(), state);

        if let Some(action) = state.current_action.take() {
            match action {
                Action::NavigateFoward => {
                    if tree.children.len() == 2 {
                        tree.children.push(Tree::empty());
                        tree.children.push(Tree::empty());
                        return;
                    }

                    tree.children.push(Tree::empty());
                    let size = tree.children.len();

                    tree.children.swap(size - 2, size - 3);
                    tree.children.swap(size - 4, size - 5);
                }
                Action::NavigateTo(index) => {
                    let size = tree.children.len();

                    tree.children.swap(size - 1, size - 3);
                    tree.children.swap(size - 2, size - 4);

                    tree.children.swap(size - 1, index);
                    tree.children.drain(index..(size - 4));
                }
                Action::NavigateBack => {
                    let size = tree.children.len();

                    tree.children.swap(size - 1, size - 3);
                    tree.children.swap(size - 2, size - 4);
                }
                Action::PopHistory => {
                    let children_len = tree.children.len();

                    if children_len > 4 {
                        tree.children.swap(children_len - 3, children_len - 5);
                        tree.children.remove(children_len - 5);
                    } else if children_len > 2 {
                        tree.children.truncate(2);
                    }
                }
                Action::ClearHistory => {
                    tree.children.truncate(2);
                }
            };
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
        let state = tree.state.downcast_ref::<State<Key>>();

        if self.header_builder.is_none() {
            self.main_header.clear_cache();
            self.secondary_header.clear_cache();
        }

        let base_layer = if let Some(transition) = state.transition.as_ref()
            && let Some(key) = state.get_previous_key()
        {
            let page_index = tree.children.len() - 3;
            let header_index = tree.children.len() - 4;

            let disc = std::mem::discriminant(key);
            let children = &mut tree.children;

            let params = PageParams {
                page: key.clone(),
                can_go_back: match transition {
                    Transition::Foward => state.history.len() > 2,
                    Transition::Back => true,
                },
            };

            self.children.get_mut(&disc).map(|page| {
                let page_header = self.header_builder.as_ref().map(|builder| {
                    let element = builder(params.clone());

                    children[header_index].diff(&element);

                    element
                });

                if page.is_empty() {
                    page.update_cache(params);
                }

                let page_element = page.take_element().unwrap();

                children[page_index].diff(&page_element);

                let LayoutResult {
                    node,
                    page: page_element,
                    header,
                } = resolve_page_layout(
                    page_header,
                    page_element,
                    header_index,
                    page_index,
                    self.width,
                    self.height,
                    &limits,
                    children,
                    renderer,
                );

                if let Some(header) = header {
                    self.secondary_header.return_element(header);
                }

                page.return_element(page_element);

                node
            })
        } else {
            None
        };

        let mut size = None;

        let main_layer = {
            let limits = if let Some(value) = &base_layer {
                size = Some(limits.resolve(self.width, self.height, value.size()));
                layout::Limits::new(Size::ZERO, size.unwrap())
            } else {
                limits
            };

            let page_index = tree.children.len() - 1;
            let header_index = tree.children.len() - 2;

            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);
            let children = &mut tree.children;

            let params = PageParams {
                page: key.clone(),
                can_go_back: state.history.len() > 1,
            };

            self.children
                .get_mut(&disc)
                .map(|page| {
                    let page_header = self.header_builder.as_ref().map(|builder| {
                        let element = builder(params.clone());

                        children[header_index].diff(&element);

                        element
                    });

                    if page.is_empty() {
                        page.update_cache(params);
                    }

                    let page_element = page.take_element().unwrap();

                    children[page_index].diff(&page_element);

                    let LayoutResult {
                        node,
                        page: page_element,
                        header,
                    } = resolve_page_layout(
                        page_header,
                        page_element,
                        header_index,
                        page_index,
                        self.width,
                        self.height,
                        &limits,
                        children,
                        renderer,
                    );

                    if let Some(header) = header {
                        self.main_header.return_element(header);
                    }

                    page.return_element(page_element);

                    node
                })
                .unwrap()
        };

        let mut nodes = Vec::with_capacity(2);
        nodes.extend(base_layer);
        nodes.push(main_layer);

        layout::Node::with_children(
            size.unwrap_or_else(|| limits.resolve(self.width, self.height, nodes[0].size())),
            nodes,
        )
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
                        } else if tree.children.len() > 3 {
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

        if let Some(header) = self.main_header.get_element_mut() {
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

        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);

        if let Some(page) = self.children.get_mut(&disc) {
            let element = page.get_element_mut().unwrap();
            let widget_state = tree.children.last_mut().unwrap();

            element.as_widget_mut().update(
                widget_state,
                event,
                layout.child(if self.main_header.is_empty() { 0 } else { 1 }),
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

        if state.transition.is_some() {
            return mouse::Interaction::default();
        }

        let children_number = tree.children.len();

        let page_state = &tree.children[children_number - 1];
        let header_state = &tree.children[children_number - 2];

        let layout = layout.children().last().unwrap();

        let header_interaction = self.main_header.get_element().map(|header| {
            header.as_widget().mouse_interaction(
                header_state,
                layout.child(0),
                cursor,
                viewport,
                renderer,
            )
        });

        header_interaction.unwrap_or_else(|| {
            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);

            self.children
                .get(&disc)
                .map(|page| {
                    let element = page.get_element().unwrap();

                    element.as_widget().mouse_interaction(
                        page_state,
                        layout.child(if self.main_header.is_empty() { 0 } else { 1 }),
                        cursor,
                        viewport,
                        renderer,
                    )
                })
                .unwrap_or_default()
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
                .map(|transition| transition.to_translation(nav_state.frame.as_ref(), &bounds))
                .unwrap_or((None, None));

            if nav_state.transition.is_some()
                && let Some(key) = nav_state.get_previous_key()
            {
                let mut clipped_viewport = clipped_viewport;
                let page_state = tree.children.get(children_len - 3).unwrap();
                let page_layout = children_layout[0].children().collect::<Vec<_>>();

                let disc = std::mem::discriminant(key);

                if let Some(page) = self.children.get(&disc) {
                    let element = page.get_element().unwrap();

                    if let Some(element) = self.secondary_header.get_element() {
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
                        page_state,
                        renderer,
                        theme,
                        style,
                        *page_layout.last().unwrap(),
                        cursor,
                        &clipped_viewport,
                    );
                }
            }

            let page_state = tree.children.get(children_len - 1).unwrap();
            let page_layout = children_layout
                .last()
                .unwrap()
                .children()
                .collect::<Vec<_>>();

            let key = nav_state.history.last().unwrap();
            let disc = std::mem::discriminant(key);

            if let Some(page) = self.children.get(&disc) {
                let element = page.get_element().unwrap();

                renderer.with_layer(clipped_viewport, |renderer| {
                    if let Some(element) = self.main_header.get_element() {
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
            .map(|transition| transition.to_translation(nav_state.frame.as_ref(), &bounds))
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

            let key = nav_state.history.last().unwrap();
            let disc = std::mem::discriminant(key);

            return self.children.get_mut(&disc).map(|page| {
                let element = page.get_element_mut().unwrap();

                let (page_state, tree_slice) = tree.children.split_last_mut().unwrap();

                let header_overlay = self.main_header.get_element_mut().map(|element| {
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

                overlay::Group::with_children(
                    header_overlay
                        .into_iter()
                        .flatten()
                        .chain(page_overlay)
                        .collect(),
                )
                .overlay()
            });
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
                .draw(tree, renderer, theme, style, layout, cursor, viewport)
        }),
        None => {
            draw_background(renderer, &background_style, *viewport);
            layer
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    };
}

impl Transition {
    fn to_translation(
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

struct LayoutResult<'a, Message, Theme, Renderer> {
    node: layout::Node,
    header: Option<Element<'a, Message, Theme, Renderer>>,
    page: Element<'a, Message, Theme, Renderer>,
}

fn resolve_page_layout<'a, Message, Theme, Renderer>(
    header: Option<Element<'a, Message, Theme, Renderer>>,
    page: Element<'a, Message, Theme, Renderer>,
    header_index: usize,
    page_index: usize,
    width: Length,
    height: Length,
    limits: &layout::Limits,
    children: &mut [Tree],
    renderer: &Renderer,
) -> LayoutResult<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    let mut elements = Vec::with_capacity(2);

    if let Some(header) = header {
        elements.push(header);
    }

    elements.push(page);
    let size = elements.len();

    let node = layout::flex::resolve(
        layout::flex::Axis::Vertical,
        renderer,
        limits,
        width,
        height,
        Padding::ZERO,
        0.0,
        iced::Alignment::Start,
        &mut elements,
        &mut children[if size == 2 { header_index } else { page_index }..page_index + 1],
    );

    LayoutResult {
        node,
        header: if size == 2 {
            Some(elements.remove(0))
        } else {
            None
        },
        page: elements.remove(0),
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
