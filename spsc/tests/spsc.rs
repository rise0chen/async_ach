use async_ach_spsc::Spsc;
use core::ops::Range;
use std::thread;
use core::time::Duration;
use futures_executor::ThreadPool;
use std::process;

const TEST_DATA: Range<usize> = 0..10000;

#[test]
fn test() {
    static SPSC: Spsc<usize, 2> = Spsc::new();
    let executor = ThreadPool::new().unwrap();
    executor.spawn_ok(async {
        let mut sender = SPSC.take_sender().unwrap();
        for i in TEST_DATA {
            sender.send(i).await;
        }
        println!("finished send");
    });
    executor.spawn_ok(async {
        let mut recver = SPSC.take_recver().unwrap();
        for i in TEST_DATA {
            assert_eq!(recver.recv().await, i);
        }
        println!("finished recv");
        thread::yield_now();
        process::exit(0);
    });
    thread::sleep(Duration::from_secs(3));
    unreachable!()
}
