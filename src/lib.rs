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
    buf: VecDeque<T>,
    capacity: usize,
    shared_state: Vec<SharedState<T>>,
}

#[derive(Debug, Clone)]
struct SharedState<T: Clone + Debug + Unpin> {
    pending: Operation<T>,
    waker: Option<Waker>,
}

#[derive(Debug, Clone)]
enum Operation<T: Clone + Debug + Unpin> {
    PushBack(T),
    PopFront,
}

struct PushBack<'a, T: Clone + Debug + Unpin> {
    buf: &'a mut ConstSizeVecDeque<T>,
    value: T,
}

impl<T: Clone + Debug + Unpin> Debug for PushBack<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "push_back -> len: {:?}, cap: {:?}, shared_state: {:?} value: {:?}",
            self.buf.len(),
            self.buf.capacity,
            self.buf.shared_state,
            self.value
        ))
    }
}

impl<'a, T: Clone + Debug + Unpin> PushBack<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>, value: T) -> Self {
        Self { buf, value }
    }
}

pub struct PopFront<'a, T: Clone + Debug + Unpin> {
    buf: &'a mut ConstSizeVecDeque<T>,
}

impl<T: Clone + Debug + Unpin> Debug for PopFront<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "pop_front -> len: {:?}, cap: {:?}, shared_state: {:?}",
            self.buf.len(),
            self.buf.capacity,
            self.buf.shared_state
        ))
    }
}

impl<'a, T: Clone + Debug + Unpin> PopFront<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>) -> Self {
        Self { buf }
    }
}

impl<T: Clone + Debug + Unpin> Future for PushBack<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let value = self.value.clone();

        if self.buf.is_full() {
            println!("Pushing a value into pending list: {value:?}");
            self.get_mut().buf.shared_state.push(SharedState {
                pending: Operation::PushBack(value),
                waker: Some(cx.waker().clone()),
            });

            Poll::Pending
        } else {
            let push_back = self.get_mut();

            push_back.buf.internal_push_back(value.clone());
            println!("{push_back:?}");

            Poll::Ready(())
        }
    }
}

impl<T: Clone + Debug + Unpin> Future for PopFront<'_, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.buf.is_empty() {
            Poll::Pending
        } else {
            let pop_front = self.get_mut();

            let element = pop_front.buf.internal_pop_front();
            println!("{pop_front:?}");

            match element {
                Some(element) => Poll::Ready(element),
                None => Poll::Pending,
            }
        }
    }
}

impl<T: Clone + Debug + Unpin> ConstSizeVecDeque<T> {
    pub fn new(len: usize) -> Self {
        Self {
            buf: VecDeque::default(),
            capacity: len,
            shared_state: Vec::new(),
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

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.buf.contains(value)
    }

    pub fn push_back(&mut self, value: T) -> impl Future<Output = ()> + '_ {
        let push_back = PushBack::new(self, value.clone());
        push_back.into_future()
    }

    fn internal_push_back(&mut self, value: T) {
        self.buf.push_back(value)
    }

    pub fn pop_front(&mut self) -> impl Future<Output = T> + '_ {
        let pop_front = PopFront::new(self);
        pop_front.into_future()
    }

    fn internal_pop_front(&mut self) -> Option<T> {
        let value = self.buf.pop_front();
        value
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
