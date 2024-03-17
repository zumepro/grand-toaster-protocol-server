use core::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};


#[derive(Default)]
pub struct Pending {}


impl Future for Pending {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
