#![allow(dead_code)]

pub enum Operations<Value> {
    PushBack(Value),
    PopFront,
}

pub trait RefVecDeque<Value> {
    fn new() -> Self
    where
        Self: Default + Sized,
    {
        Self::default()
    }

    fn push_back(&mut self, _value: Value);
    fn pop_front(&mut self) -> Value;
}

pub trait ActualVecDeque<Value> {
    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }

    fn push_back(&mut self, _value: Value);
    fn pop_front(&mut self) -> Value;
}

#[derive(Default)]
struct SampleStruct;

impl SampleStruct {}

impl<Value> RefVecDeque<Value> for SampleStruct {
    fn push_back(&mut self, _value: Value) {
        todo!()
    }
    fn pop_front(&mut self) -> Value {
        todo!()
    }
}

impl<Value> ActualVecDeque<Value> for SampleStruct {
    fn push_back(&mut self, _value: Value) {
        todo!()
    }
    fn pop_front(&mut self) -> Value {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_invariants<Value, R, I>(reference: &R, implementation: &I) -> bool
    where
        R: RefVecDeque<Value>,
        I: ActualVecDeque<Value>,
    {
        todo!()
    }

    #[test]
    fn test_random_ops() {
        // TODO: Generate a random list of operations.
        let ops: Vec<Operations<u32>> = Vec::new();

        let mut reference = <SampleStruct as RefVecDeque<u32>>::new();
        let mut implementation = <SampleStruct as ActualVecDeque<u32>>::new();

        for op in ops {
            match op {
                Operations::PushBack(value) => {
                    RefVecDeque::push_back(&mut reference, value);
                    ActualVecDeque::push_back(&mut implementation, value);
                }
                Operations::PopFront => {
                    assert_eq!(
                        RefVecDeque::<u32>::pop_front(&mut reference),
                        ActualVecDeque::<u32>::pop_front(&mut implementation)
                    )
                }
            }

            assert!(check_invariants::<u32, _, _>(&reference, &implementation))
        }
    }
}
