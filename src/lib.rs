#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::{select, time};

    use super::*;

    #[tokio::test]
    async fn async_push_back() {
        // create a new async vecdeque.
        let mut tester: ConstSizeVecDeque<10>;

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
        assert!(tester.contains(&11))
    }
}
