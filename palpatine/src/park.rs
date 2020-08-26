use ::std::{
    cell::RefCell,
    marker::PhantomData,
    result,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{RawWaker, RawWakerVTable, Waker},
};

use ::parking_lot::{Condvar, Mutex};

type Error = ();
pub type Result<T> = result::Result<T, Error>;

thread_local! {
    static CACHED_PARKER: RefCell<Option<Parker>> = RefCell::new(None);
}

const EMPTY: usize = 0b_0000;
const PARKED: usize = 0b_0001;
const NOTIFIED: usize = 0b_0010;

#[repr(transparent)]
pub struct Parker {
    inner: Arc<Inner>,
}
#[derive(Copy, Clone)]
pub struct CachedParker<'a> {
    _anchor: PhantomData<&'a std::rc::Rc<()>>,
}

#[repr(transparent)]
pub struct Unparker {
    inner: Arc<Inner>,
}

struct Inner {
    state: AtomicUsize,
    mutex: Mutex<()>,
    cond: Condvar,
}

impl Parker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: AtomicUsize::new(EMPTY),
                mutex: Mutex::new(()),
                cond: Condvar::new(),
            }),
        }
    }
    #[must_use]
    pub fn unparker(&self) -> Unparker {
        let inner = self.inner.clone();
        Unparker { inner }
    }

    /// # Errors
    /// Can't error, result is for mapping
    pub fn park(&mut self) -> Result<()> {
        self.inner.park();
        Ok(())
    }
}

impl<'a> CachedParker<'a> {
    fn new() -> Self {
        Self {
            _anchor: PhantomData,
        }
    }
    fn with<R, F: FnOnce(&RefCell<Option<Parker>>) -> R>(f: F) -> R {
        CACHED_PARKER
            .try_with(|parker| f(parker))
            .expect("couldn't access thread_local storage")
    }

    #[must_use]
    pub fn current<'b>() -> CachedParker<'b> {
        Self::with(|cache| {
            let mut cached = cache.borrow_mut();
            *cached = match cached.take() {
                Some(parker) => Some(parker),
                None => Some(Parker::new()),
            }
        });
        CachedParker::new()
    }

    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn unparker(self) -> Unparker {
        Self::with(|parker| {
            parker
                .borrow()
                .as_ref()
                .expect("parker should be set at unparker")
                .unparker()
        })
    }

    #[allow(clippy::unused_self)]
    pub fn park(self) {
        Self::with(|parker| {
            parker
                .borrow()
                .as_ref()
                .expect("parker should be set at parking operation")
                .inner
                .park()
        })
    }
}

impl Unparker {
    pub fn unpark(&self) {
        self.inner.unpark()
    }

    #[must_use]
    pub fn into_waker(self) -> Waker {
        unsafe { Waker::from_raw(self.inner.to_raw_waker()) }
    }
}

impl Inner {
    const VTABLE: RawWakerVTable =
        RawWakerVTable::new(Self::clone, Self::wake, Self::wake_by_ref, Self::drop_waker);

    unsafe fn to_raw_waker(self: Arc<Self>) -> RawWaker {
        RawWaker::new(Arc::into_raw(self) as *const (), &Self::VTABLE)
    }

    unsafe fn from_raw(ptr: *const ()) -> Arc<Inner> {
        Arc::from_raw(ptr as *const Inner)
    }

    fn park(&self) {
        use Ordering::SeqCst;

        if self
            .state
            .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_ok()
        {
            return;
        }

        let mut g = self.mutex.lock();

        match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
            Ok(_) => {}
            Err(NOTIFIED) => {
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");
                return;
            }
            Err(actual) => panic!("inconsistent park state; actual = {}", actual),
        }

        while self
            .state
            .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_err()
        {
            self.cond.wait(&mut g);
        }
    }

    fn unpark(&self) {
        match self.state.swap(NOTIFIED, Ordering::SeqCst) {
            EMPTY | NOTIFIED => return,
            PARKED => {}
            _ => panic!("inconsistent state in unpark"),
        }

        drop(self.mutex.lock());
        assert!(self.cond.notify_one(), "expected a listening parker");
    }

    unsafe fn clone(raw: *const ()) -> RawWaker {
        let unparker = Inner::from_raw(raw);
        std::mem::forget(unparker.clone()); // increment ref count
        unparker.to_raw_waker()
    }

    unsafe fn drop_waker(raw: *const ()) {
        let _ = Inner::from_raw(raw);
    }

    unsafe fn wake(raw: *const ()) {
        let unparker = Inner::from_raw(raw);
        unparker.unpark();
    }

    unsafe fn wake_by_ref(raw: *const ()) {
        let unparker = Inner::from_raw(raw);
        unparker.unpark();
        std::mem::forget(unparker); // we don't own the the waker
    }
}

impl Default for Parker {
    fn default() -> Self {
        Self::new()
    }
}
