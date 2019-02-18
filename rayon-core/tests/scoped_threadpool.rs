extern crate crossbeam_utils;
extern crate rayon_core;

#[macro_use]
extern crate scoped_tls;

use crossbeam_utils::thread;
use rayon_core::ThreadPoolBuilder;

#[derive(PartialEq, Eq, Debug)]
struct Local(i32);

scoped_thread_local!(static LOCAL: Local);

#[test]
fn scoped_tls_missing() {
    LOCAL.set(&Local(42), || {
        let pool = ThreadPoolBuilder::new()
            .build()
            .expect("thread pool created");

        // `LOCAL` is not set in the pool.
        pool.install(|| {
            assert!(!LOCAL.is_set());
        });
    });
}

#[test]
fn scoped_tls_threadpool() {
    LOCAL.set(&Local(42), || {
        LOCAL.with(|x| {
            thread::scope(|scope| {
                let pool = ThreadPoolBuilder::new()
                    .spawn(move |thread| {
                        scope
                            .builder()
                            .spawn(move |_| {
                                // Borrow the same local value in the thread pool.
                                LOCAL.set(x, || thread.run())
                            })
                            .map(|_| ())
                    })
                    .expect("thread pool created");

                // The pool matches our local value.
                pool.install(|| {
                    assert!(LOCAL.is_set());
                    LOCAL.with(|y| {
                        assert_eq!(x, y);
                    });
                });

                // If we change our local value, the pool is not affected.
                LOCAL.set(&Local(-1), || {
                    pool.install(|| {
                        assert!(LOCAL.is_set());
                        LOCAL.with(|y| {
                            assert_eq!(x, y);
                        });
                    });
                });
            })
            .expect("scope threads ok");
            // `thread::scope` will wait for the threads to exit before returning.
        });
    });
}
