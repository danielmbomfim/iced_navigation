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

use crate::base::{NavigatorPage, NavigatorState};

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

    fn history_len(&self) -> usize {
        self.history.len()
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
    tabs_element: Option<
        Box<
            dyn for<'b> Fn(PageParams<Key>, &Vec<Key>) -> Element<'a, Message, Theme, Renderer>
                + 'a,
        >,
    >,
    cache: [Option<Element<'a, Message, Theme, Renderer>>; 2],
    children:
        IndexMap<Discriminant<Key>, NavigatorPage<'a, PageParams<Key>, Message, Theme, Renderer>>,
    on_navigation_end: Option<Box<dyn Fn(Option<Key>, Key) -> Message + 'a>>,
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
            tabs_element: None,
            children: IndexMap::new(),
            cache: [None, None],
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

    pub fn tabs_widget(
        mut self,
        fun: impl Fn(PageParams<Key>, &Vec<Key>) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        self.tabs_element = Some(Box::new(fun));

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
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Key>>();

        operation.custom(self.id.as_ref(), layout.bounds(), state);
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > self.children.len() + 1 {
            if let Some(item) = tree.children.pop() {
                tree.children.truncate(self.children.len());
                tree.children.push(item);
            }
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
        let mut items = Vec::with_capacity(2);

        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);
        let page_index = self.children.get_index_of(&disc).unwrap();
        let children_len = tree.children.len();

        let (page, moved) = {
            if let Some(NavigatorPage::Closure(builder)) = self.children.get(&disc) {
                let params = PageParams {
                    current_page: key.clone(),
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
            }
        };

        items.push(page.unwrap());

        if let Some(builder) = self.tabs_element.as_ref() {
            let params = PageParams {
                current_page: key.clone(),
                can_go_back: state.history.len() > 1,
            };

            let el = builder(params, &self.pages);

            tree.children[children_len - 1].diff(&el);

            items.push(el);
        }

        tree.children.swap(page_index, children_len - 2);

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
            &mut tree.children[children_len - 2..],
        );

        if let Mode::Top = self.mode {
            let mut children = node.children().to_vec();

            let header_position = Point::new(children[0].bounds().x, children[0].bounds().y);
            let page_position = Point::new(
                children[0].bounds().x,
                children[0].bounds().y + children[1].bounds().height,
            );

            children[0].move_to_mut(page_position);
            children[1].move_to_mut(header_position);

            node = Node::with_children(node.size(), children);
        }

        if moved {
            self.children
                .insert(disc, NavigatorPage::Direct(items.remove(0)));
            self.cache[0] = None;
        } else {
            self.cache[0] = Some(items.remove(0));
        }

        if !items.is_empty() {
            self.cache[1] = Some(items.remove(0));
        }

        tree.children.swap(page_index, children_len - 2);

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

                if let Some(on_navigation_end) = self.on_navigation_end.as_ref() {
                    shell.publish(on_navigation_end(
                        state.get_previous_key().cloned(),
                        state.history.last().cloned().unwrap(),
                    ));
                }
                return;
            }
        }

        let children_len = tree.children.len();
        let key = state.history.last().unwrap();
        let disc = std::mem::discriminant(key);
        let page_index = self.children.get_index_of(&disc).unwrap();
        let children_layout: Vec<_> = layout.children().collect();

        match self.cache[0].as_mut() {
            Some(child) => {
                child.as_widget_mut().update(
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
            None => {
                if let Some(NavigatorPage::Direct(child)) = self.children.get_mut(&disc) {
                    child.as_widget_mut().update(
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
            }
        };

        if let Some(child) = self.cache[1].as_mut() {
            child.as_widget_mut().update(
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

        let page_interaction = match self.cache[0].as_ref() {
            Some(child) => Some(child.as_widget().mouse_interaction(
                &tree.children[page_index],
                children_layout[0],
                cursor,
                viewport,
                renderer,
            )),
            None => {
                if let Some(NavigatorPage::Direct(child)) = self.children.get(&disc) {
                    Some(child.as_widget().mouse_interaction(
                        &tree.children[page_index],
                        children_layout[0],
                        cursor,
                        viewport,
                        renderer,
                    ))
                } else {
                    None
                }
            }
        };

        let tabs_interaction = if let Some(child) = self.cache[1].as_ref() {
            Some(child.as_widget().mouse_interaction(
                &tree.children[children_len - 1],
                children_layout[1],
                cursor,
                viewport,
                renderer,
            ))
        } else {
            None
        };

        page_interaction.or(tabs_interaction).unwrap_or_default()
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

            if let Some(child) = self.cache[1].as_ref() {
                child.as_widget().draw(
                    &tree.children[children_len - 1],
                    renderer,
                    theme,
                    style,
                    children_layout[1],
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
                        children_layout[0],
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
                            children_layout[0],
                            cursor,
                            &clipped_viewport,
                        );
                    }
                }
            };
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

            let (tabs_cache, cache) = self.cache.split_last_mut().unwrap();
            let (tabs_state, tree_slice) = if tabs_cache.is_some() {
                let (tabs_state, slice) = tree.children.split_last_mut().unwrap();

                (Some(tabs_state), slice)
            } else {
                (None, tree.children.as_mut_slice())
            };

            let tabs_overlay = tabs_cache.as_mut().map(|element| {
                element.as_widget_mut().overlay(
                    tabs_state.unwrap(),
                    children_layout[1],
                    renderer,
                    &clipped_viewport,
                    translation,
                )
            });

            let page_overlay = match cache[0].as_mut() {
                Some(element) => element.as_widget_mut().overlay(
                    &mut tree_slice[page_index],
                    children_layout[0],
                    renderer,
                    &clipped_viewport,
                    translation,
                ),
                None => {
                    if let Some(NavigatorPage::Direct(element)) = self.children.get_mut(&disc) {
                        element.as_widget_mut().overlay(
                            &mut tree_slice[page_index],
                            children_layout[0],
                            renderer,
                            &clipped_viewport,
                            translation,
                        )
                    } else {
                        None
                    }
                }
            };

            return Some(
                overlay::Group::with_children(
                    tabs_overlay
                        .into_iter()
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
