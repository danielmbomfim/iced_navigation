use iced::Element;

#[cfg(feature = "drawer")]
pub mod drawer_navigator;
pub mod operations;
#[cfg(feature = "stack")]
pub mod stack_navigator;
#[cfg(feature = "tabs")]
pub mod tabs_navigator;

#[allow(dead_code)]
pub(crate) enum NavigatorElementSource<'a, Params, Message, Theme, Renderer = iced::Renderer> {
    Direct(Element<'a, Message, Theme, Renderer>),
    Closure(Box<dyn Fn(Params) -> Element<'a, Message, Theme, Renderer> + 'a>),
    None,
}

impl<'a, Params, Message, Theme, Renderer>
    From<NavigatorElementSource<'a, Params, Message, Theme, Renderer>>
    for NavigatorElement<'a, Params, Message, Theme, Renderer>
{
    fn from(val: NavigatorElementSource<'a, Params, Message, Theme, Renderer>) -> Self {
        NavigatorElement {
            source: val,
            cache: None,
            taken: false,
        }
    }
}

pub(crate) trait NavigatorState {
    type Key;

    fn request_update(&mut self);

    fn get_previous_key(&self) -> Option<&Self::Key>;

    fn navigate(&mut self, page: Self::Key);

    fn go_back(&mut self);

    fn pop_history(&mut self);

    fn clear_history(&mut self);
}

pub(crate) struct NavigatorElement<'a, Params, Message, Theme, Renderer = iced::Renderer> {
    source: NavigatorElementSource<'a, Params, Message, Theme, Renderer>,
    cache: Option<Element<'a, Message, Theme, Renderer>>,
    taken: bool,
}

impl<'a, Params, Message, Theme, Renderer> NavigatorElement<'a, Params, Message, Theme, Renderer> {
    pub fn empty() -> Self {
        Self {
            cache: None,
            source: NavigatorElementSource::None,
            taken: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_none() && !matches!(self.source, NavigatorElementSource::Direct(_))
    }

    pub fn clear_cache(&mut self) {
        self.cache = None;
    }

    pub fn take_element(&mut self) -> Option<Element<'a, Message, Theme, Renderer>> {
        

        self.cache.take().or_else(|| {
            if let NavigatorElementSource::Direct(element) =
                std::mem::replace(&mut self.source, NavigatorElementSource::None)
            {
                self.taken = true;
                return Some(element);
            }

            None
        })
    }

    pub fn return_element(&mut self, element: Element<'a, Message, Theme, Renderer>) {
        if !self.taken {
            self.cache = Some(element);
            return;
        }

        let _ = std::mem::replace(&mut self.source, NavigatorElementSource::Direct(element));
        self.taken = false;
    }

    pub fn update_cache(&mut self, params: Params) {
        if let NavigatorElementSource::Closure(builder) = &self.source {
            self.cache = Some(builder(params));
        };
    }

    pub fn get_element(&self) -> Option<&Element<'a, Message, Theme, Renderer>> {
        if let NavigatorElementSource::Direct(element) = &self.source {
            return Some(element);
        };

        self.cache.as_ref()
    }

    pub fn get_element_mut(&mut self) -> Option<&mut Element<'a, Message, Theme, Renderer>> {
        if let NavigatorElementSource::Direct(element) = &mut self.source {
            return Some(element);
        };

        self.cache.as_mut()
    }
}
