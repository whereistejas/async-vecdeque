#![allow(dead_code)]

use std::{mem::MaybeUninit, ptr};

pub struct ConstSizeVecDeque<T, const N: usize>
where
    T: PartialEq,
{
    len: usize,
    buffer: [MaybeUninit<T>; N],
}

impl<T, const N: usize> ConstSizeVecDeque<T, N>
where
    T: PartialEq,
{
    const CAPACITY: usize = N;

    pub fn new() -> Self {
        Self {
            len: 0,
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    pub fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    async fn push_back(&mut self, value: T) {
        let len = self.len();
        let mut_ptr = self.buffer.as_mut_ptr() as *mut T;

        unsafe {
            ptr::write(mut_ptr.add(len), value);
        }

        self.set_len(len + 1)
    }

    async fn pop_front(&mut self) -> T {
        todo!()
    }

    // fn contains(&self, value: T) -> bool {
    //     self.buffer.contains(&Some(value))
    // }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::{select, time};

    use super::*;

    #[tokio::test]
    async fn async_push_back() {
        // create a new async vecdeque.
        let mut tester = ConstSizeVecDeque::<_, 10>::new();

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

        // // assert that 11 has been pushed into the buffer.
        // assert!(tester.contains(11))
    }
}
