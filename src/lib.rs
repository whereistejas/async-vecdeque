use std::{
    collections::VecDeque,
    future::{self, Future},
};

pub struct ConstSizeVecDeque<T: Clone> {
    buf: VecDeque<T>,
    len: usize,
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
        if self.is_full() {
            todo!()
        } else {
            self.buf.push_back(value);
            future::ready(())
        }
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
