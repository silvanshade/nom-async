use futures::{
    prelude::*,
    task::{Context, Poll},
};
use nom::{Err, IResult};
use std::{borrow::Borrow, convert::AsRef, ops::AddAssign, pin::Pin};

/// A [Stream](futures::stream::Stream) constructed from a nom streaming parser
pub struct NomStream<'a, I, O, T, E, S>
where
    I: ?Sized + ToOwned,
    <I as ToOwned>::Owned: for<'i> AddAssign<&'i I>,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>>,
{
    stream: S,
    parser: Box<dyn 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>>,
    buffer: <I as ToOwned>::Owned,
}

impl<'a, I, O, T, E, S> NomStream<'a, I, O, T, E, S>
where
    I: ?Sized + ToOwned,
    <I as ToOwned>::Owned: for<'i> AddAssign<&'i I>,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>>,
{
    /// Construct a new [NomStream] from a stream and parser
    pub fn new<F>(stream: S, parser: F) -> Self
    where
        F: 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>,
        <I as ToOwned>::Owned: Default,
    {
        let buffer = Default::default();
        Self::new_with_buffer(stream, parser, buffer)
    }

    /// Construct a new [NomStream] from a stream, parser, and buffer
    pub fn new_with_buffer<F>(stream: S, parser: F, buffer: <I as ToOwned>::Owned) -> Self
    where
        F: 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>,
    {
        let parser = Box::new(parser) as Box<_>;
        NomStream { stream, parser, buffer }
    }
}

impl<'a, I, O, T, E, S> Stream for NomStream<'a, I, O, T, E, S>
where
    I: ?Sized + ToOwned,
    <I as ToOwned>::Owned: for<'i> AddAssign<&'i I> + Unpin,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>> + Unpin,
{
    type Item = O;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<O>> {
        match (self.parser)(self.buffer.borrow()) {
            Ok((i, o)) => {
                self.buffer = i.to_owned();
                cx.waker().clone().wake();
                Poll::Ready(Some(o))
            },
            Err(err) => match err {
                Err::Incomplete(_needed) => match Pin::new(&mut self.stream).poll_next(cx) {
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Ready(Some(res)) => match res {
                        Ok(item) => {
                            self.buffer += item.as_ref();
                            cx.waker().clone().wake();
                            Poll::Pending
                        },
                        Err(_err) => Poll::Ready(None),
                    },
                    Poll::Pending => Poll::Pending,
                },
                Err::Error(_error) => Poll::Ready(None),
                Err::Failure(_failure) => Poll::Ready(None),
            },
        }
    }
}
