use futures::task::{Context, Poll};
use iou::SubmissionQueueEvent;
use ringbahn::drive::demo::{DemoDriver, driver};
use ringbahn::drive::{Completion, Drive};
use std::cell::RefCell;
use std::pin::Pin;

pub struct Driver {
    demo: RefCell<DemoDriver<'static>>,
}

pub fn spawn_driver() -> &'static Driver {
    Box::leak(Box::new(Driver{
        demo: RefCell::new(driver()),
    }))
}

impl Drive for &'static Driver {
    fn poll_prepare<'cx>(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'cx>,
        prepare: impl FnOnce(SubmissionQueueEvent<'_>, &mut Context<'cx>) -> Completion<'cx>,
    ) -> Poll<Completion<'cx>> {
        Pin::new(&mut *self.demo.borrow_mut()).poll_prepare(ctx, prepare)
    }

    fn poll_submit(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        eager: bool,
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut *self.demo.borrow_mut()).poll_submit(cx, eager)
    }
}

impl Drive for &'_ mut &'static Driver {
    fn poll_prepare<'cx>(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'cx>,
        prepare: impl FnOnce(SubmissionQueueEvent<'_>, &mut Context<'cx>) -> Completion<'cx>,
    ) -> Poll<Completion<'cx>> {
        Pin::new(&mut *self.demo.borrow_mut()).poll_prepare(ctx, prepare)
    }

    fn poll_submit(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        eager: bool,
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut *self.demo.borrow_mut()).poll_submit(cx, eager)
    }
}
