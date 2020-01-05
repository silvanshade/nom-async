use futures::{
    prelude::*,
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
    stream: S,
    parser: Box<dyn 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>>,
    buffer: B,
}

impl<'a, B, I, O, T, E, S> NomStream<'a, B, I, O, T, E, S>
where
    B: for<'i> AddAssign<&'i I>,
    I: ?Sized,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>>,
{
    /// Construct a new [NomStream] from a stream and parser
    pub fn new<F>(stream: S, parser: F) -> Self
    where
        F: 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>,
        B: Default,
    {
        let buffer = Default::default();
        Self::new_with_buffer(stream, parser, buffer)
    }

    /// Construct a new [NomStream] from a stream, parser, and buffer
    pub fn new_with_buffer<F>(stream: S, parser: F, buffer: B) -> Self
    where
        F: 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>,
    {
        let parser = Box::new(parser) as Box<_>;
        NomStream { stream, parser, buffer }
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
