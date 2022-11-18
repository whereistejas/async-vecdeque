#![allow(dead_code)]
#![feature(into_future)]

use std::{
    collections::VecDeque,
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

pub struct ConstSizeVecDeque<T: Clone> {
    buf: VecDeque<T>,
    len: usize,
}

struct PushBack<T> {
    buf: VecDeque<T>,
    value: T,
}

impl<T> PushBack<T> {
    fn new(buf: VecDeque<T>, value: T) -> Self {
        Self { buf, value }
    }
}

impl<T> Future for PushBack<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // if self.is_full() {
        //     todo!()
        // } else {
        //     self.buf.push_back(value);
        //     future::ready(())
        // }

        todo!()
    }
}
pub struct PopFront<T> {
    buf: VecDeque<T>,
}

impl<T> PopFront<T> {
    fn new(buf: VecDeque<T>) -> Self {
        Self { buf }
    }
}

impl<T> Future for PopFront<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

impl<T: Clone> ConstSizeVecDeque<T> {
    pub fn new(len: usize) -> Self {
        Self {
            buf: VecDeque::default(),
            len,
        }
    }

    pub fn is_full(&self) -> bool {
        self.len < self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn push_back(&mut self, value: T) -> impl Future<Output = ()> {
        let push_back = PushBack::new(self.buf.clone(), value);
        push_back.into_future()
    }

    pub async fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            todo!()
        }
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
            tester.push_back(item).await;
        }

        // on the 11th item, the task should block.
        let fut = tester.push_back(11);
        let res = time::timeout(Duration::from_secs(1), fut).await;
        assert!(res.is_err());
    }
}

// use async_trait::async_trait;
// #[async_trait]
// pub trait PushBack<T: Send> {
//     async fn push_back(&mut self, _value: T)
//     where
//         T: 'async_trait,
//     {
//         todo!()
//     }
// }

// impl<T: Send> PushBack<T> for ConstSizeVecDeque<T> {}

// #[async_trait]
// pub trait PopFront<T> {
//     async fn pop_front(&mut self) -> Option<T> {
//         todo!()
//     }
// }

// impl<T> PopFront<T> for ConstSizeVecDeque<T> {}
