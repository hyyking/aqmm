use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use crate::{
    io::driver::{self, context},
    park::CachedParker,
};

thread_local! {
    static QUEUE: Queue = Queue::new();
}

pub fn spawn<F: Future + 'static>(f: F) {
    let schedule = |task| {
        QUEUE.with(|queue| {
            queue.push(task);
        })
    };
    let (task, _) = async_task::spawn_local(f, schedule, ());
    task.schedule();
}

pub struct Executor {
    driver: driver::Driver,
}

struct Queue {
    inner: RefCell<Vec<async_task::Task<()>>>,
}
impl Queue {
    const fn new() -> Self {
        Self {
            inner: RefCell::new(Vec::new()),
        }
    }
    pub(crate) fn push(&self, task: async_task::Task<()>) {
        self.inner.borrow_mut().push(task)
    }
    pub(crate) fn pop(&self) -> Option<async_task::Task<()>> {
        self.inner.borrow_mut().pop()
    }
}

impl Executor {
    /// # Errors
    /// Fails if the I/O driver couldn't be created
    pub fn new() -> std::io::Result<Self> {
        driver::Driver::new().map(|driver| Self { driver })
    }
    pub fn block_on<F: Future + 'static>(&mut self, mut f: F) -> F::Output {
        let mut pinned = unsafe { Pin::new_unchecked(&mut f) };
        let parker = CachedParker::current();
        let waker = parker.unparker().into_waker();
        let cx = &mut Context::from_waker(&waker);

        enter(self, |scheduler, queue| 'outer: loop {
            if let Poll::Ready(output) = pinned.as_mut().poll(cx) {
                return output;
            }

            for _ in 0..64 {
                if let Some(task) = queue.pop() {
                    let _: bool = task.run();
                } else {
                    scheduler.driver.turn(None).expect("unable to turn driver");
                    continue 'outer;
                }
            }
            scheduler
                .driver
                .turn(Some(Duration::from_millis(0)))
                .expect("unable to turn driver");
        })
    }
}

fn enter<F, R>(exec: &mut Executor, mut f: F) -> R
where
    F: FnMut(&mut Executor, &Queue) -> R,
{
    QUEUE.with(|queue| context::enter(exec.driver.handle(), || f(exec, queue)))
}
