use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}

impl<T> Stream for Pin<T>
where
    T: std::ops::DerefMut + Unpin,
    T::Target: Stream,
{
    type Item = <T::Target as Stream>::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().as_mut().poll_next(cx)
    }
}

impl<T> Stream for &mut T
where
    T: Stream + Unpin + ?Sized
{
    type Item = T::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut **self).poll_next(cx)
    }
}
