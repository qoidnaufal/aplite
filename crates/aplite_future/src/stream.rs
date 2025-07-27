use std::future::Future;
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

/// A wrapper for impl [`Stream`] type, since [`Future`] trait can't be applied to T: Stream
pub trait StreamExt: Stream {
    fn stream(&mut self) -> StreamWrapper<'_, Self> {
        StreamWrapper { inner: self }
    }
}

pub struct StreamWrapper<'a, T>
where
    T: ?Sized + Stream
{
    inner: &'a mut T
}

impl<T> Future for StreamWrapper<'_, T>
where
    T: ?Sized + Stream + Unpin,
{
    type Output = Option<T::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl<T: ?Sized + Stream + Unpin> StreamExt for T {}
