use std::cell::RefCell;

use super::Handle;

thread_local! {
    static CONTEXT: RefCell<Option<Handle>> = RefCell::default();
}

struct DropGuard(Option<Handle>);

impl Drop for DropGuard {
    fn drop(&mut self) {
        CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = self.0.take();
        });
    }
}

pub(crate) fn enter<R>(new: Handle, f: impl FnOnce() -> R) -> R {
    let _guard = CONTEXT.with(|ctx| {
        let old = ctx.borrow_mut().replace(new);
        DropGuard(old)
    });

    f()
}

pub(crate) fn current() -> Option<Handle> {
    CONTEXT.with(|ctx| ctx.borrow().as_ref().map(Clone::clone))
}
