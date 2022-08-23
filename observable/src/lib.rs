use futures::stream::Stream;
use std::future::Future;
use std::marker::Unpin;
use std::pin::Pin;
use std::sync::mpsc;
use std::task::{Context, Poll};

pub struct Observable<T, F> {
    receiver: mpsc::Receiver<T>,
    future: F,
    is_completed: bool,
}

impl<T, F> Observable<T, F>
where
    F: Future<Output = ()>,
{
    pub fn create<S>(source: S) -> Self
    where
        S: FnOnce(Observer<T>) -> F,
    {
        let (sender, receiver) = mpsc::channel();
        let observer = Observer { sender };
        let future = source(observer);
        Self {
            future,
            receiver,
            is_completed: false,
        }
    }
}

impl<T, F> Stream for Observable<T, F>
where
    F: Future<Output = ()>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (future, receiver, is_completed) = unsafe {
            let project = self.get_unchecked_mut();
            (
                Pin::new_unchecked(&mut project.future),
                &mut project.receiver,
                &mut project.is_completed,
            )
        };
        if *is_completed {
            Poll::Ready(receiver.try_recv().ok())
        } else {
            match future.poll(cx) {
                Poll::Ready(_) => {
                    *is_completed = true;
                    Poll::Ready(receiver.try_recv().ok())
                }
                Poll::Pending => match receiver.try_recv() {
                    Ok(value) => Poll::Ready(Some(value)),
                    Err(_) => Poll::Pending,
                },
            }
        }
    }
}

impl<T, F> Unpin for Observable<T, F> {}

pub struct Observer<T> {
    sender: mpsc::Sender<T>,
}

impl<T> Observer<T> {
    pub fn on_next(&self, value: T) {
        self.sender.send(value).unwrap();
    }
}
