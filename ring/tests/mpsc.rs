use async_ach_ring::Ring;
use core::time::Duration;
use futures_executor::ThreadPool;
use std::collections::BTreeSet;
use std::process;
use std::thread;

const TEST_TIMES: usize = 10000;

#[test]
fn test() {
    static RING: Ring<usize, 2, 2, 1> = Ring::new();
    let executor = ThreadPool::new().unwrap();
    executor.spawn_ok(async {
        // Producer_1
        for i in 0..TEST_TIMES {
            RING.push(i).await;
        }
        println!("Producer_1 finished");
    });
    executor.spawn_ok(async {
        // Producer_2
        for i in TEST_TIMES..2 * TEST_TIMES {
            RING.push(i).await;
        }
        println!(" Producer_2 finished");
    });
    executor.spawn_ok(async {
        // Cunsumer_1
        let mut data_set: BTreeSet<usize> = (0..2 * TEST_TIMES).collect();
        for _ in 0..2 * TEST_TIMES {
            let data = RING.pop().await;
            assert!(data_set.remove(&data));
        }
        println!("Cunsumer_1 finished");
        thread::yield_now();
        assert!(data_set.is_empty());
        process::exit(0);
    });
    thread::sleep(Duration::from_secs(3));
    unreachable!()
}
