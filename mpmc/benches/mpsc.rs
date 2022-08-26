#[macro_use]
extern crate criterion;

use criterion::Criterion;
use futures_util::sink::SinkExt as _;
use futures_util::stream::StreamExt as _;
use std::sync::Arc;
use std::thread;

struct TestMpsc;
impl TestMpsc {
    const SENDER_NUM: usize = 10;
    const TEST_NUM: usize = 10000;

    fn ach_heapless(c: &mut Criterion) {
        use async_ach_mpmc::heapless as mpsc;
        c.bench_function("ach_heapless", |b| {
            b.iter(|| {
                let ch = mpsc::Mpmc::<usize, { Self::SENDER_NUM }, { Self::SENDER_NUM }, 1>::new();
                let ch = Arc::new(ch);
                thread::scope(|scope| {
                    for _ in 0..Self::SENDER_NUM {
                        scope.spawn(|| {
                            let ch = ch.clone();
                            futures_executor::block_on(async {
                                for i in 0..Self::TEST_NUM {
                                    ch.sender().send(i).await;
                                }
                            })
                        });
                    }
                    futures_executor::block_on(async {
                        for _ in 0..Self::SENDER_NUM * Self::TEST_NUM {
                            let _ = ch.recver().recv().await;
                        }
                    });
                });
            })
        });
    }
    #[cfg(feature = "alloc")]
    fn ach_heap(c: &mut Criterion) {
        c.bench_function("ach_heap", |b| {
            use async_ach_mpmc::heap as mpsc;
            b.iter(|| {
                let (tx,rx) = mpsc::channel::<usize, { Self::SENDER_NUM }, { Self::SENDER_NUM }, 1>();
                let ch = Arc::new(tx);
                thread::scope(|scope| {
                    for _ in 0..Self::SENDER_NUM {
                        scope.spawn(|| {
                            let ch = ch.clone();
                            futures_executor::block_on(async {
                                for i in 0..Self::TEST_NUM {
                                    ch.send(i).await;
                                }
                            })
                        });
                    }
                    futures_executor::block_on(async {
                        for _ in 0..Self::SENDER_NUM * Self::TEST_NUM {
                            let _ = rx.recv().await;
                        }
                    });
                });
            })
        });
    }
    fn tokio_mpsc(c: &mut Criterion) {
        c.bench_function("tokio_mpsc", |b| {
            b.iter(|| {
                let (tx, mut rx) = tokio::sync::mpsc::channel(Self::SENDER_NUM);
                let ch = Arc::new(tx);
                thread::scope(|scope| {
                    for _ in 0..Self::SENDER_NUM {
                        scope.spawn(|| {
                            let ch = ch.clone();
                            futures_executor::block_on(async {
                                for i in 0..Self::TEST_NUM {
                                    ch.send(i).await.unwrap();
                                }
                            })
                        });
                    }
                    futures_executor::block_on(async {
                        for _ in 0..Self::SENDER_NUM * Self::TEST_NUM {
                            let _ = rx.recv().await.unwrap();
                        }
                    });
                });
            })
        });
    }
    fn futures_channel(c: &mut Criterion) {
        c.bench_function("futures_channel", |b| {
            b.iter(|| {
                let (ch, mut rx) = futures_channel::mpsc::channel(Self::SENDER_NUM);
                thread::scope(|scope| {
                    for _ in 0..Self::SENDER_NUM {
                        scope.spawn(|| {
                            let mut ch = ch.clone();
                            futures_executor::block_on(async {
                                for i in 0..Self::TEST_NUM {
                                    ch.send(i).await.unwrap();
                                }
                            })
                        });
                    }
                    futures_executor::block_on(async {
                        for _ in 0..Self::SENDER_NUM * Self::TEST_NUM {
                            let _ = rx.next().await.unwrap();
                        }
                    });
                });
            })
        });
    }
    fn async_channel(c: &mut Criterion) {
        c.bench_function("async_channel", |b| {
            b.iter(|| {
                let (tx, rx) = async_channel::bounded(Self::SENDER_NUM);
                let ch = Arc::new(tx);
                thread::scope(|scope| {
                    for _ in 0..Self::SENDER_NUM {
                        scope.spawn(|| {
                            let ch = ch.clone();
                            futures_executor::block_on(async {
                                for i in 0..Self::TEST_NUM {
                                    ch.send(i).await.unwrap();
                                }
                            })
                        });
                    }
                    futures_executor::block_on(async {
                        for _ in 0..Self::SENDER_NUM * Self::TEST_NUM {
                            let _ = rx.recv().await.unwrap();
                        }
                    });
                });
            })
        });
    }
    fn flume(c: &mut Criterion) {
        c.bench_function("flume", |b| {
            b.iter(|| {
                let (tx, rx) = flume::bounded(Self::SENDER_NUM);
                let ch = Arc::new(tx);
                thread::scope(|scope| {
                    for _ in 0..Self::SENDER_NUM {
                        scope.spawn(|| {
                            let ch = ch.clone();
                            futures_executor::block_on(async {
                                for i in 0..Self::TEST_NUM {
                                    ch.send_async(i).await.unwrap();
                                }
                            })
                        });
                    }
                    futures_executor::block_on(async {
                        for _ in 0..Self::SENDER_NUM * Self::TEST_NUM {
                            let _ = rx.recv_async().await.unwrap();
                        }
                    });
                });
            })
        });
    }
}

criterion_group!(
    test_mpsc,
    TestMpsc::ach_heapless,
    TestMpsc::ach_heap,
    TestMpsc::tokio_mpsc,
    TestMpsc::futures_channel,
    TestMpsc::async_channel,
    TestMpsc::flume
);
criterion_main!(test_mpsc);
