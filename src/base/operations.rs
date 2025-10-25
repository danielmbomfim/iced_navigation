use std::{hash::Hash, marker::PhantomData};

use iced::{
    Rectangle, Task,
    advanced::widget::{Id, Operation, operate},
};

use crate::base::{self, NavigatorState};

fn clear_history_op<T, Key>(target: Option<Id>) -> impl Operation<T>
where
    Key: 'static + Eq + Hash + Clone + Send,
{
    struct ClearHistory<Key> {
        target: Option<Id>,
        p: PhantomData<Key>,
    }

    impl<T, Key> Operation<T> for ClearHistory<Key>
    where
        Key: 'static + Eq + Hash + Clone + Send,
    {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self)
        }

        fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            #[cfg(feature = "stack")]
            if let Some(value) = state.downcast_mut::<base::stack_navigator::State<Key>>() {
                value.request_update();

                if id.is_some_and(|id| self.target.as_ref().is_some_and(|target| target != id)) {
                    return;
                }

                value.clear_history();
            }
        }
    }

    ClearHistory {
        target,
        p: PhantomData::<Key>,
    }
}

fn pop_history_op<T, Key>(target: Option<Id>) -> impl Operation<T>
where
    Key: 'static + Eq + Hash + Clone + Send,
{
    struct PopHistory<Key> {
        target: Option<Id>,
        p: PhantomData<Key>,
    }

    impl<T, Key> Operation<T> for PopHistory<Key>
    where
        Key: 'static + Eq + Hash + Clone + Send,
    {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self)
        }

        fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            #[cfg(feature = "stack")]
            if let Some(value) = state.downcast_mut::<base::stack_navigator::State<Key>>() {
                value.request_update();

                if id.is_some_and(|id| self.target.as_ref().is_some_and(|target| target != id)) {
                    return;
                }

                value.pop_history();
            }
        }
    }

    PopHistory {
        target,
        p: PhantomData::<Key>,
    }
}

fn go_back_op<T, Key>(target: Option<Id>) -> impl Operation<T>
where
    Key: 'static + Eq + Hash + Clone + Send,
{
    struct GoBack<Key> {
        target: Option<Id>,
        p: PhantomData<Key>,
    }

    impl<T, Key> Operation<T> for GoBack<Key>
    where
        Key: 'static + Eq + Hash + Clone + Send,
    {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self)
        }

        fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            #[cfg(feature = "stack")]
            if let Some(value) = state.downcast_mut::<base::stack_navigator::State<Key>>() {
                value.request_update();

                if id.is_some_and(|id| self.target.as_ref().is_some_and(|target| target != id)) {
                    return;
                }

                value.go_back();
            }
        }
    }

    GoBack {
        target,
        p: PhantomData::<Key>,
    }
}

fn navigate_op<T, Key>(page: Key, target: Option<Id>) -> impl Operation<T>
where
    Key: 'static + Eq + Hash + Clone + Send,
{
    struct Navigate<Key> {
        target: Option<Id>,
        page: Option<Key>,
    }

    impl<T, Key> Operation<T> for Navigate<Key>
    where
        Key: 'static + Eq + Hash + Clone + Send,
    {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self)
        }

        fn custom(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
            #[cfg(feature = "stack")]
            if let Some(value) = state.downcast_mut::<base::stack_navigator::State<Key>>() {
                value.request_update();

                if id.is_some_and(|id| self.target.as_ref().is_some_and(|target| target != id)) {
                    return;
                }

                value.navigate(self.page.take().unwrap());
            }
        }
    }

    Navigate {
        target,
        page: Some(page),
    }
}

pub fn navigate<T, P>(page: P) -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(navigate_op::<T, P>(page, None))
}

pub fn navigate_by_id<T, P>(page: P, target: Id) -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(navigate_op::<T, P>(page, Some(target)))
}

pub fn go_back<T, P>() -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(go_back_op::<T, P>(None))
}

pub fn go_back_by_id<T, P>(target: Id) -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(go_back_op::<T, P>(Some(target)))
}

pub fn clear_history<T, P>() -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(clear_history_op::<T, P>(None))
}

pub fn clear_history_by_id<T, P>(target: Id) -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(clear_history_op::<T, P>(Some(target)))
}

pub fn pop_history<T, P>() -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(pop_history_op::<T, P>(None))
}

pub fn pop_history_by_id<T, P>(target: Id) -> Task<T>
where
    P: 'static + Eq + Hash + Clone + Send,
    T: 'static + Send,
{
    operate(pop_history_op::<T, P>(Some(target)))
}
