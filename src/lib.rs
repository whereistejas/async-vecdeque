#![feature(test)]
use std::{
    collections::VecDeque,
    future::{self, Future},
};

extern crate test;
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
    use super::*;
    use test::Bencher;

    use std::fs::{remove_file, OpenOptions};
    use std::io::Write;
    use std::path::Path;

    #[bench]
    fn single_write(b: &mut Bencher) {
        let path = Path::new("/tmp/random_test_file");

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();

        let array: [u64; 8] = rand::random();

        b.iter(|| {
            for _ in 0..10_000 {
                file.write_all(format!("{:?}\n", array).as_bytes()).unwrap();
            }
        });

        remove_file(path).unwrap();
    }
}
