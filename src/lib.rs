#![allow(dead_code)]
#![feature(into_future)]

use std::{
    collections::VecDeque,
    fmt::Debug,
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll, Waker},
};

// TODO: Get rid of this `Clone` and all the clones that follow it.
#[derive(Debug, Clone)]
pub struct ConstSizeVecDeque<T: Clone + Debug + Unpin> {
    buffer: VecDeque<T>,
    capacity: usize,
    pending_push: Vec<(Option<Waker>, T)>,
    pending_pop: Vec<Option<Waker>>,
}

struct PushBack<'a, T: Clone + Debug + Unpin> {
    buffer: &'a mut ConstSizeVecDeque<T>,
    value: T,
}

impl<T: Clone + Debug + Unpin> Debug for PushBack<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "push_back -> len: {:?}, cap: {:?}, pending_push: {:?} value: {:?}",
            self.buffer.len(),
            self.buffer.capacity,
            self.buffer.pending_push,
            self.value
        ))
    }
}

impl<'a, T: Clone + Debug + Unpin> PushBack<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>, value: T) -> Self {
        Self { buffer: buf, value }
    }
}

pub struct PopFront<'a, T: Clone + Debug + Unpin> {
    buffer: &'a mut ConstSizeVecDeque<T>,
}

impl<T: Clone + Debug + Unpin> Debug for PopFront<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "pop_front -> len: {:?}, cap: {:?}, pending_pop: {:?}",
            self.buffer.len(),
            self.buffer.capacity,
            self.buffer.pending_pop
        ))
    }
}

impl<'a, T: Clone + Debug + Unpin> PopFront<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>) -> Self {
        Self { buffer: buf }
    }
}

impl<T: Clone + Debug + Unpin> Future for PushBack<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let value = self.value.clone();

        if self.buffer.is_full() {
            self.get_mut()
                .buffer
                .pending_push
                .push((Some(cx.waker().clone()), value));

            Poll::Pending
        } else {
            let push_back = self.get_mut();

            push_back.buffer.buffer.push_back(value);

            println!("{push_back:?}");

            Poll::Ready(())
        }
    }
}

impl<T: Clone + Debug + Unpin> Future for PopFront<'_, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.buffer.is_empty() {
            self.get_mut()
                .buffer
                .pending_pop
                .push(Some(cx.waker().clone()));

            Poll::Pending
        } else {
            let pop_front = self.get_mut();

            let element = pop_front.buffer.buffer.pop_front();
            println!("{pop_front:?}");

            match element {
                Some(element) => Poll::Ready(element),
                None => {
                    pop_front.buffer.pending_pop.push(Some(cx.waker().clone()));
                    Poll::Pending
                }
            }
        }
    }
}

impl<T: Clone + Debug + Unpin> ConstSizeVecDeque<T> {
    pub fn new(len: usize) -> Self {
        Self {
            buffer: VecDeque::default(),
            capacity: len,
            pending_push: Vec::new(),
            pending_pop: Vec::new(),
        }
    }

    fn is_full(&self) -> bool {
        self.capacity == self.buffer.len()
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.buffer.contains(value)
    }

    pub fn push_back(&mut self, value: T) -> impl Future<Output = ()> + '_ {
        let push_back = PushBack::new(self, value.clone());
        push_back.into_future()
    }
    pub fn pop_front(&mut self) -> impl Future<Output = T> + '_ {
        let pop_front = PopFront::new(self);
        pop_front.into_future()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::time;

    use super::*;

    #[tokio::test]
    async fn async_push_back() {
        // create a new async vecdeque.
        let mut tester = ConstSizeVecDeque::new(10);

        // fill it with 10 items
        for item in 1..11 {
            tester.push_back(item).await;
        }

        // on the 11th item, the task should block.
        let fut = tester.push_back(11);
        let res = time::timeout(Duration::from_secs(1), fut).await;
        assert!(res.is_err());

        // pop an item.
        let _ = tester.pop_front().await;

        // assert that 11 has been pushed into the buffer.
        assert!(tester.contains(&10))
    }
}
