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
#[derive(Debug)]
pub struct ConstSizeVecDeque<T: Clone + Debug + Unpin + PartialEq> {
    buffer: VecDeque<T>,
    capacity: usize,
    pending_push: Vec<(Option<Waker>, T)>,
    pending_pop: Vec<Option<Waker>>,
}

struct PushBack<'a, T: Clone + Debug + Unpin + PartialEq> {
    buffer: &'a mut ConstSizeVecDeque<T>,
    value: T,
}

impl<T: Clone + Debug + Unpin + PartialEq> Debug for PushBack<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "push_back -> len: {:?}, cap: {:?}, pending_push: {:?}, pending_pop: {:?}, value: {:?}",
            self.buffer.len(),
            self.buffer.capacity,
            self.buffer.pending_push,
            self.buffer.pending_pop,
            self.value
        ))
    }
}

impl<'a, T: Clone + Debug + Unpin + PartialEq> PushBack<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>, value: T) -> Self {
        Self { buffer: buf, value }
    }
}

pub struct PopFront<'a, T: Clone + Debug + Unpin + PartialEq> {
    buffer: &'a mut ConstSizeVecDeque<T>,
}

impl<T: Clone + Debug + Unpin + PartialEq> Debug for PopFront<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "pop_front -> len: {:?}, cap: {:?}, pending_push: {:?}, pending_pop: {:?}",
            self.buffer.len(),
            self.buffer.capacity,
            self.buffer.pending_push,
            self.buffer.pending_pop
        ))
    }
}

impl<'a, T: Clone + Debug + Unpin + PartialEq> PopFront<'a, T> {
    fn new(buf: &'a mut ConstSizeVecDeque<T>) -> Self {
        Self { buffer: buf }
    }
}

impl<T: Clone + Debug + Unpin + PartialEq> Future for PushBack<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let push_back = self.get_mut();
        println!("{push_back:?}");

        let value = push_back.value.clone();

        for waker in push_back.buffer.pending_pop.iter_mut() {
            if let Some(waker) = waker.take() {
                waker.wake()
            }
        }

        if push_back.buffer.is_full() {
            // Don't add the same value twice.
            if push_back
                .buffer
                .pending_push
                .iter()
                .all(|(_, pending_value)| pending_value != &value)
            {
                println!("Pushing {value:?} into pending list.");
                push_back
                    .buffer
                    .pending_push
                    .push((Some(cx.waker().clone()), value));
                println!("{push_back:?}");
            } else {
                for (waker, value) in push_back.buffer.pending_push.iter() {
                    println!("{value:?}");
                    if waker.is_none() {
                        panic!()
                    }
                }
            }

            Poll::Pending
        } else {
            while let Some((idx, (waker, value))) =
                push_back.buffer.pending_push.iter().enumerate().next()
            {
                println!("Trying to push value: {value:?}");
                if waker.is_none() {
                    push_back.buffer.buffer.push_back(value.clone());
                    push_back.buffer.pending_push.remove(idx);
                }
            }

            push_back.buffer.buffer.push_back(value);

            Poll::Ready(())
        }
    }
}

impl<T: Clone + Debug + Unpin + PartialEq> Future for PopFront<'_, T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pop_front = self.get_mut();

        println!("{pop_front:?}");
        if pop_front.buffer.is_empty() {
            pop_front.buffer.pending_pop.push(Some(cx.waker().clone()));

            Poll::Pending
        } else {
            while let Some((idx, waker)) = pop_front.buffer.pending_pop.iter().enumerate().next() {
                if waker.is_none() {
                    let element = pop_front.buffer.buffer.pop_front();
                    pop_front.buffer.pending_pop.remove(idx);
                    return match element {
                        Some(element) => Poll::Ready(element),
                        None => Poll::Pending,
                    };
                }
            }

            let element = pop_front.buffer.buffer.pop_front();
            println!("Updated length post popping: {:?}", pop_front.buffer.len());

            match element {
                Some(element) => {
                    // NOTE: It is possible that a PushBack operation is waiting because the list is full.
                    // Let's repoll the pending the operations.
                    for (waker, value) in pop_front.buffer.pending_push.iter_mut() {
                        println!("Waking task for: {value:?}");
                        if let Some(waker) = waker.take() {
                            waker.wake()
                        }
                    }

                    Poll::Ready(element)
                }
                None => {
                    pop_front.buffer.pending_pop.push(Some(cx.waker().clone()));
                    Poll::Pending
                }
            }
        }
    }
}

impl<T: Clone + Debug + Unpin + PartialEq> ConstSizeVecDeque<T> {
    pub fn push_back(&mut self, value: T) -> impl Future<Output = ()> + '_ {
        let push_back = PushBack::new(self, value);
        push_back.into_future()
    }

    pub fn pop_front(&mut self) -> impl Future<Output = T> + '_ {
        let pop_front = PopFront::new(self);
        pop_front.into_future()
    }

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
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::{select, time};

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
        // NOTE: This causes the future to be polled multiple times.
        let result = select! {
            _ = tester.push_back(11) => Ok(()),
            _ = time::sleep(Duration::from_secs(1)) => Err(()),
        };

        assert!(result.is_err());

        // pop an item.
        let item = tester.pop_front().await;
        assert_eq!(item, 1);

        // assert that 11 has been pushed into the buffer.
        let mut stop = 0;
        loop {
            if tester.pending_push.is_empty() {
                break;
            } else {
                stop += 1;
                if stop <= 5 {
                    time::sleep(Duration::from_secs(1)).await
                } else {
                    panic!("Timed out! {tester:?}")
                }
            }
        }

        assert!(tester.contains(&11))
    }
}
