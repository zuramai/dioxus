use std::cell::Cell;

use futures_util::Future;

use crate::{
    component::{Component, ComponentFn, Dummy, IntoComponent},
    element::Element,
    scopes::{Scope, ScopeState},
};

pub trait AnyProps {
    fn as_ptr(&self) -> *const ();
    fn render<'a>(&'a self, bump: &'a ScopeState) -> Element<'a>;
    unsafe fn memoize(&self, other: &dyn AnyProps) -> bool;
}

pub(crate) struct VComponentProps<P, F: Future<Output = ()> = Dummy> {
    pub render_fn: ComponentFn<P, F>,
    pub memo: unsafe fn(&P, &P) -> bool,
    pub props: *const P,
}

impl VComponentProps<()> {
    pub fn new_empty(render_fn: Component<()>) -> Self {
        Self {
            render_fn: render_fn.into_component(),
            memo: <() as PartialEq>::eq,
            props: std::ptr::null_mut(),
        }
    }
}

impl<P> VComponentProps<P> {
    pub(crate) fn new(
        render_fn: Component<P>,
        memo: unsafe fn(&P, &P) -> bool,
        props: *const P,
    ) -> Self {
        Self {
            render_fn: render_fn.into_component(),
            memo,
            props,
        }
    }
}

impl<P> AnyProps for VComponentProps<P> {
    fn as_ptr(&self) -> *const () {
        &self.props as *const _ as *const ()
    }

    // Safety:
    // this will downcast the other ptr as our swallowed type!
    // you *must* make this check *before* calling this method
    // if your functions are not the same, then you will downcast a pointer into a different type (UB)
    unsafe fn memoize(&self, other: &dyn AnyProps) -> bool {
        let real_other: &P = &*(other.as_ptr() as *const _ as *const P);
        let real_us: &P = &*(self.as_ptr() as *const _ as *const P);
        (self.memo)(real_us, real_other)
    }

    fn render<'a>(&'a self, scope: &'a ScopeState) -> Element<'a> {
        // Make sure the scope ptr is not null
        // self.props.state.set(scope);

        let scope = Scope {
            props: unsafe { &*self.props },
            scope,
        };

        // Call the render function directly
        // todo: implement async
        match self.render_fn {
            ComponentFn::Sync(f) => f(scope),
            ComponentFn::Async(_) => todo!(),
        }
    }
}
