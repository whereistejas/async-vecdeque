#![allow(dead_code)]
#![feature(into_future)]

use std::{
    collections::VecDeque,
    fmt::Debug,
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

// TODO: Get rid of this `Clone` and all the clones that follow it.
#[derive(Debug, Clone)]
pub struct ConstSizeVecDeque<T: Clone + Debug + Unpin> {
    buf: VecDeque<T>,
    capacity: usize,
}

#[derive(Debug)]
struct PushBack<'a, T: Clone + Debug + Unpin> {
    buf: &'a mut ConstSizeVecDeque<T>,
    value: T,
}

impl<'a, T: Clone + Debug + Unpin> PushBack<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>, value: T) -> Self {
        Self { buf, value }
    }
}

impl<T: Clone + Debug + Unpin> Future for PushBack<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Buffer len: {:?}", self.buf.len());

        if self.buf.is_full() {
            Poll::Pending
        } else {
            let value = self.value.clone();
            self.get_mut().buf.internal_push_back(value.clone());
            println!("Added: {:?}", value);

            Poll::Ready(())
        }
    }
}

#[derive(Debug)]
pub struct PopFront<'a, T: Clone + Debug + Unpin> {
    buf: &'a mut ConstSizeVecDeque<T>,
}

impl<'a, T: Clone + Debug + Unpin> PopFront<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>) -> Self {
        Self { buf }
    }
}

impl<T: Clone + Debug + Unpin> Future for PopFront<'_, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.buf.is_empty() {
            Poll::Ready(None)
        } else {
            todo!()
        }
    }
}

impl<T: Clone + Debug + Unpin> ConstSizeVecDeque<T> {
    pub fn new(len: usize) -> Self {
        Self {
            buf: VecDeque::default(),
            capacity: len,
        }
    }

    fn is_full(&self) -> bool {
        self.capacity == self.buf.len()
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn push_back(&mut self, value: T) -> impl Future<Output = ()> + '_ {
        let push_back = PushBack::new(self, value.clone());
        println!("PushBack: {push_back:?}");
        push_back.into_future()
    }

    fn internal_push_back(&mut self, value: T) {
        self.buf.push_back(value)
    }

    pub fn pop_front(&mut self) -> impl Future<Output = Option<T>> + '_ {
        let pop_front = PopFront::new(self);
        println!("PopFront: {pop_front:?}");
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
        // create a new async vecdeque
        let mut tester = ConstSizeVecDeque::new(10);

        // fill it with 10 items
        for item in 0..10 {
            println!("Item: {item:?}");
            tester.push_back(item).await;
        }

        // on the 11th item, the task should block.
        let fut = tester.push_back(11);
        let res = time::timeout(Duration::from_secs(1), fut).await;
        assert!(res.is_err());
    }
}
