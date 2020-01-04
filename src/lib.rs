#![deny(clippy::all)]
#![deny(missing_docs)]

//! Async adapters for nom streaming parsers

use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use nom::{Err, IResult};
use std::{borrow::Borrow, convert::AsRef, ops::AddAssign, pin::Pin};

/// A [Stream](futures::stream::Stream) constructed from a nom streaming parser
pub struct NomStream<'a, B, I, O, T, E, S>
where
    B: for<'i> AddAssign<&'i I>,
    I: ?Sized,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>>,
{
    parser: Box<dyn 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>>,
    buffer: B,
    stream: S,
}

impl<'a, B, I, O, T, E, S> NomStream<'a, B, I, O, T, E, S>
where
    B: for<'i> AddAssign<&'i I>,
    I: ?Sized,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>>,
{
    /// Construct a new [NomStream](NomStream) from a parser, buffer, and stream
    pub fn new<F>(parser: F, buffer: B, stream: S) -> Self
    where
        F: 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>,
    {
        let parser = Box::new(parser) as Box<_>;
        NomStream { parser, buffer, stream }
    }
}

impl<'a, B, I, O, T, E, S> Stream for NomStream<'a, B, I, O, T, E, S>
where
    B: for<'i> AddAssign<&'i I> + Borrow<I> + for<'i> From<&'i I> + Unpin,
    I: ?Sized + Unpin,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>> + Unpin,
{
    type Item = O;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<O>> {
        match (self.parser)(self.buffer.borrow()) {
            Ok((i, o)) => {
                self.buffer = i.into();
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
