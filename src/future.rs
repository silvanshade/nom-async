use futures::{
    prelude::*,
    task::{Context, Poll},
};
use nom::{Err, IResult};
use std::{borrow::Borrow, convert::AsRef, ops::AddAssign, pin::Pin};

/// A [Future](futures::future::Future) constructed from a nom streaming parser
pub struct NomFuture<'a, B, I, O, T, E, S>
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

impl<'a, B, I, O, T, E, S> NomFuture<'a, B, I, O, T, E, S>
where
    B: for<'i> AddAssign<&'i I>,
    I: ?Sized,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>>,
{
    /// Construct a new [NomFuture](NomFuture) from a parser, buffer, and stream
    pub fn new<F>(parser: F, buffer: B, stream: S) -> Self
    where
        F: 'a + for<'i> Fn(&'i I) -> IResult<&'i I, O>,
    {
        let parser = Box::new(parser) as Box<_>;
        NomFuture { parser, buffer, stream }
    }
}

impl<'a, B, I, O, T, E, S> Future for NomFuture<'a, B, I, O, T, E, S>
where
    B: for<'i> AddAssign<&'i I> + Borrow<I> + for<'i> From<&'i I> + Unpin,
    I: ?Sized + Unpin,
    T: AsRef<I>,
    S: Stream<Item = Result<T, E>> + Unpin,
{
    type Output = O;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<O> {
        match (self.parser)(self.buffer.borrow()) {
            Ok((i, o)) => {
                self.buffer = i.into();
                cx.waker().clone().wake();
                Poll::Ready(o)
            },
            Err(err) => match err {
                Err::Incomplete(_needed) => match Pin::new(&mut self.stream).poll_next(cx) {
                    Poll::Ready(None) => panic!(),
                    Poll::Ready(Some(res)) => match res {
                        Ok(item) => {
                            self.buffer += item.as_ref();
                            cx.waker().clone().wake();
                            Poll::Pending
                        },
                        Err(_err) => panic!(),
                    },
                    Poll::Pending => Poll::Pending,
                },
                Err::Error(_error) => panic!(),
                Err::Failure(_failure) => panic!(),
            },
        }
    }
}
