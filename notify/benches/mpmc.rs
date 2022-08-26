#[macro_use]
extern crate criterion;

mod ring;

use criterion::Criterion;
use std::sync::Arc;
use std::thread;

struct TestMpmc<const T: usize, const R: usize, const N: usize>;
impl<const T: usize, const R: usize, const N: usize> TestMpmc<T, R, N> {
    fn ach_notify(c: &mut Criterion) {
        c.bench_function("ach_notify", |b| {
            use ring::ach_notify::Ring;
            b.iter(|| {
                let ch = Ring::<usize, T, T, R>::new();
                let ch = Arc::new(ch);
                thread::scope(|scope| {
                    for _ in 0..T {
                        scope.spawn(|| {
                            futures_executor::block_on(async {
                                for i in 0..N {
                                    ch.push(i).await;
                                }
                            })
                        });
                    }
                    for _ in 0..R {
                        scope.spawn(|| {
                            futures_executor::block_on(async {
                                for _ in 0..N * T / R {
                                    let _ = ch.pop().await;
                                }
                            })
                        });
                    }
                });
            })
        });
    }
    fn event_listener(c: &mut Criterion) {
        c.bench_function("event_listener", |b| {
            use ring::event_listener::Ring;
            b.iter(|| {
                let ch = Ring::<usize, T, T, R>::new();
                let ch = Arc::new(ch);
                thread::scope(|scope| {
                    for _ in 0..T {
                        scope.spawn(|| {
                            futures_executor::block_on(async {
                                for i in 0..N {
                                    ch.push(i).await;
                                }
                            })
                        });
                    }
                    for _ in 0..R {
                        scope.spawn(|| {
                            futures_executor::block_on(async {
                                for _ in 0..N * T / R {
                                    let _ = ch.pop().await;
                                }
                            })
                        });
                    }
                });
            })
        });
    }
    fn tokio_notify(c: &mut Criterion) {
        c.bench_function("tokio_notify", |b| {
            use ring::tokio_notify::Ring;
            b.iter(|| {
                let ch = Ring::<usize, T, T, R>::new();
                let ch = Arc::new(ch);
                thread::scope(|scope| {
                    for _ in 0..T {
                        scope.spawn(|| {
                            futures_executor::block_on(async {
                                for i in 0..N {
                                    ch.push(i).await;
                                }
                            })
                        });
                    }
                    for _ in 0..R {
                        scope.spawn(|| {
                            futures_executor::block_on(async {
                                for _ in 0..N * T / R {
                                    let _ = ch.pop().await;
                                }
                            })
                        });
                    }
                });
            })
        });
    }
}

type TestMpmc10_10_10000 = TestMpmc<10, 1, 10000>;

criterion_group!(
    test_mpmc,
    TestMpmc10_10_10000::ach_notify,
    TestMpmc10_10_10000::tokio_notify,
    TestMpmc10_10_10000::event_listener,
);
criterion_main!(test_mpmc);
