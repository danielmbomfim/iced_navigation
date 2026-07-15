use std::hash::Hash;
use std::mem::Discriminant;

use iced::advanced::graphics::core::window;
use iced::advanced::layout::Node;
use iced::advanced::overlay;
use iced::advanced::widget::Operation;
use iced::widget::Id;
use iced::{
    Element, Event, Length, Rectangle, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
        widget::{Tree, tree},
    },
};
use iced::{Padding, Point, Vector};
use indexmap::IndexMap;

use crate::widgets::{NavigatorElement, NavigatorElementSource, NavigatorState};

type TabsBuilderFn<'a, Key, Message, Theme, Renderer> =
    dyn for<'b> Fn(PageParams<Key>, &Vec<Key>) -> Element<'a, Message, Theme, Renderer> + 'a;

type OnNavigationEnd<'a, Key, Message> = dyn Fn(Option<Key>, Key) -> Message + 'a;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct State<Key: Eq + Hash> {
    pub(crate) history: Vec<Key>,
    pub(crate) previous_page: Option<Key>,
    pub(crate) pending_update: bool,
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
        self.previous_page = None;
    }

    fn go_back(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.previous_page = self.history.pop();
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

#[derive(Debug, Clone, Copy)]
pub struct PageParams<Key> {
    pub current_page: Key,
    pub can_go_back: bool,
}

pub struct TabsNavigator<'a, Key, Message, Renderer = iced::Renderer>
where
    Key: Eq + Hash + Clone,
{
    id: Option<Id>,
    width: Length,
    height: Length,
    home_page: Key,
    mode: Mode,
    pages: Vec<Key>,
    tabs_builder: Option<Box<TabsBuilderFn<'a, Key, Message, Theme, Renderer>>>,
    tabs_cache: NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    children: IndexMap<
        Discriminant<Key>,
        NavigatorElement<'a, PageParams<Key>, Message, Theme, Renderer>,
    >,
    on_navigation_end: Option<Box<OnNavigationEnd<'a, Key, Message>>>,
}

impl<'a, Key, Message, Renderer> TabsNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone,
{
    pub fn new(home_page: Key) -> Self {
        Self {
            id: None,
            width: Length::Fill,
            height: Length::Fill,
            mode: Mode::Bottom,
            tabs_builder: None,
            children: IndexMap::new(),
            tabs_cache: NavigatorElement::empty(),
            on_navigation_end: None,
            pages: Vec::new(),
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

    pub fn tabs_widget(
        mut self,
        fun: impl Fn(PageParams<Key>, &Vec<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.tabs_builder = Some(Box::new(fun));

        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;

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
    for TabsNavigator<'a, Key, Message, Renderer>
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
        })
    }

    fn children(&self) -> Vec<Tree> {
        let mut children: Vec<_> = self.children.iter().map(|(_, _)| Tree::empty()).collect();

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
        let children_layout: Vec<_> = layout.children().collect();

        if let Some(tabs) = self.tabs_cache.get_element_mut() {
            operation.traverse(&mut |operation| {
                tabs.as_widget_mut().operate(
                    &mut tree.children[children_len - 1],
                    children_layout[1],
                    renderer,
                    operation,
                );
            });
        }

        if let Some(page) = self.children.get_mut(&disc) {
            operation.traverse(&mut |operation| {
                page.get_element_mut().unwrap().as_widget_mut().operate(
                    &mut tree.children[page_index],
                    children_layout[0],
                    renderer,
                    operation,
                );
            });
        }

        operation.custom(self.id.as_ref(), layout.bounds(), state);
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > self.children.len() + 1
            && let Some(item) = tree.children.pop()
        {
            tree.children.truncate(self.children.len());
            tree.children.push(item);
        }

        while tree.children.len() < self.children.len() + 1 {
            tree.children.insert(tree.children.len() - 2, Tree::empty());
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let state = tree.state.downcast_mut::<State<Key>>();

        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);
        let page_index = self.children.get_index_of(&disc).unwrap();
        let children = &mut tree.children;

        if self.tabs_builder.is_none() {
            self.tabs_cache.clear_cache();
        }

        let params = PageParams {
            current_page: key.clone(),
            can_go_back: state.history.len() > 1,
        };

        self.children
            .get_mut(&disc)
            .map(|page| {
                let tabs_element = self.tabs_builder.as_ref().map(|builder| {
                    let element = builder(params.clone(), &self.pages);

                    children[children_len - 1].diff(&element);

                    element
                });

                if page.is_empty() {
                    page.update_cache(params);
                }

                let page_element = page.take_element().unwrap();

                children[page_index].diff(&page_element);

                let mut items = Vec::with_capacity(2);
                items.push(page_element);
                items.extend(tabs_element);

                children.swap(page_index, children_len - 2);

                let mut node = layout::flex::resolve(
                    layout::flex::Axis::Vertical,
                    renderer,
                    &limits,
                    self.width,
                    self.height,
                    Padding::ZERO,
                    0.0,
                    iced::Alignment::Start,
                    &mut items,
                    &mut children[children_len - 2..],
                );

                page.return_element(items.remove(0));

                if !items.is_empty() {
                    self.tabs_cache.return_element(items.remove(0));
                }

                if let Mode::Top = self.mode {
                    let mut children = node.children().to_vec();

                    let header_position =
                        Point::new(children[0].bounds().x, children[0].bounds().y);
                    let page_position = Point::new(
                        children[0].bounds().x,
                        children[0].bounds().y + children[1].bounds().height,
                    );

                    children[0].move_to_mut(page_position);
                    children[1].move_to_mut(header_position);

                    node = Node::with_children(node.size(), children);
                }

                children.swap(page_index, children_len - 2);

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

        if let Event::Window(window::Event::RedrawRequested(_)) = event
            && state.pending_update
        {
            state.pending_update = false;
            shell.invalidate_layout();
            shell.request_redraw();

            if let Some(on_navigation_end) = self.on_navigation_end.as_ref() {
                shell.publish(on_navigation_end(
                    state.get_previous_key().cloned(),
                    state.history.last().cloned().unwrap(),
                ));
            }
            return;
        }

        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);
        let page_index = self.children.get_index_of(&disc).unwrap();
        let children_layout: Vec<_> = layout.children().collect();

        if let Some(page) = self.children.get_mut(&disc) {
            let element = page.get_element_mut().unwrap();

            element.as_widget_mut().update(
                &mut tree.children[page_index],
                event,
                children_layout[0],
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if let Some(tabs) = self.tabs_cache.get_element_mut() {
            tabs.as_widget_mut().update(
                &mut tree.children[children_len - 1],
                event,
                children_layout[1],
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
        let children_layout: Vec<_> = layout.children().collect();

        let interaction = self
            .children
            .get(&disc)
            .map(|page| {
                let element = page.get_element().unwrap();

                element.as_widget().mouse_interaction(
                    &tree.children[page_index],
                    children_layout[0],
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .or_else(|| {
                self.tabs_cache.get_element().map(|tabs| {
                    tabs.as_widget().mouse_interaction(
                        &tree.children[children_len - 1],
                        children_layout[1],
                        cursor,
                        viewport,
                        renderer,
                    )
                })
            });

        interaction.unwrap_or_default()
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
            let children_layout: Vec<_> = layout.children().collect();

            if let Some(page) = self.children.get(&disc) {
                let element = page.get_element().unwrap();

                element.as_widget().draw(
                    &tree.children[page_index],
                    renderer,
                    theme,
                    style,
                    children_layout[0],
                    cursor,
                    &clipped_viewport,
                );
            }

            if let Some(tabs) = self.tabs_cache.get_element() {
                tabs.as_widget().draw(
                    &tree.children[children_len - 1],
                    renderer,
                    theme,
                    style,
                    children_layout[1],
                    cursor,
                    &clipped_viewport,
                );
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

            if state.pending_update {
                return None;
            }

            let key = state.history.last().unwrap();
            let disc = std::mem::discriminant(key);
            let page_index = self.children.get_index_of(&disc).unwrap();
            let children_layout: Vec<_> = layout.children().collect();

            let (tabs_state, tree_slice) = if !self.tabs_cache.is_empty() {
                let (tabs_state, slice) = tree.children.split_last_mut().unwrap();

                (Some(tabs_state), slice)
            } else {
                (None, tree.children.as_mut_slice())
            };

            let tabs_overlay = self.tabs_cache.get_element_mut().map(|tabs| {
                tabs.as_widget_mut().overlay(
                    tabs_state.unwrap(),
                    children_layout[1],
                    renderer,
                    &clipped_viewport,
                    translation,
                )
            });

            self.children.get_mut(&disc).map(|page| {
                let element = page.get_element_mut().unwrap();

                let page_overlay = element.as_widget_mut().overlay(
                    &mut tree_slice[page_index],
                    children_layout[0],
                    renderer,
                    &clipped_viewport,
                    translation,
                );

                overlay::Group::with_children(
                    tabs_overlay
                        .into_iter()
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

impl<'a, Key, Message, Renderer> From<TabsNavigator<'a, Key, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Key: 'static + Eq + Hash + Clone,
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(tabs: TabsNavigator<'a, Key, Message, Renderer>) -> Self {
        Self::new(tabs)
    }
}

pub fn tabs_navigator<'a, Key, Message, Renderer>(
    home_page: Key,
) -> TabsNavigator<'a, Key, Message, Renderer>
where
    Key: Eq + Hash + Clone,
    Renderer: iced::advanced::Renderer,
{
    TabsNavigator::new(home_page)
}
