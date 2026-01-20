// These traits and impls are required so that we can send the
// rule fragment closures across thread boundaries and keep the
// compiler happy. But they're ugly. So they're getting a home
// here so they're out of sight. Thankfully they should never change

use crate::EnrichedContext as Context;

pub trait SendableFragment: Fn(&Context) -> bool {
    fn clone_boxed(&self) -> Box<dyn SendableFragment + Send + Sync + 'static>;
}

impl<T> SendableFragment for T
where
    T: 'static + Clone + Sync + Send + Fn(&Context) -> bool,
{
    fn clone_boxed(&self) -> Box<dyn SendableFragment + Send + Sync + 'static> {
        Box::new(T::clone(self))
    }
}

impl Clone for Box<dyn SendableFragment + Send + Sync + 'static> {
    fn clone(&self) -> Self {
        self.as_ref().clone_boxed()
    }
}
