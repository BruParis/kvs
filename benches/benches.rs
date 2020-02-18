#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark};
use kvs::{KVEngine, KVStore, SledKVEngine};
use rand::prelude::*;
use std::iter;
use tempfile::TempDir;

fn engine_write(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs_set",
        |b, _| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    (KVStore::open(temp_dir.path()).unwrap(), temp_dir)
                },
                |(mut store, _temp_dir)| {
                    for i in 1..(1 << 2) {
                        store.set(format!("{}", i), "bench".to_string()).unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        },
        iter::once(()),
    )
    .with_function("sled_set", |b, _| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                (SledKVEngine::open(temp_dir.path()).unwrap(), temp_dir)
            },
            |(mut store, _temp_dir)| {
                for i in 1..(1 << 2) {
                    store.set(format!("{}", i), "bench".to_string()).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
    c.bench("kvs_write", bench);
}

fn engine_read(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs_read",
        |b, _| {
            let temp_dir = TempDir::new().unwrap();
            let mut store = KVStore::open(temp_dir.path()).unwrap();
            for i in 1..(1 << 2) {
                store.set(format!("{}", i), "bench".to_string()).unwrap();
            }
            let mut rng = StdRng::from_seed([0; 32]);
            b.iter(|| {
                for _i in 1..(1 << 2) {
                    let rand_num = rng.gen::<u8>();
                    store.get(format!("{}", rand_num)).unwrap();
                }
            })
        },
        iter::once(()),
    )
    .with_function("sled_read", |b, _| {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledKVEngine::open(temp_dir.path()).unwrap();
        for i in 1..(1 << 2) {
            store.set(format!("{}", i), "bench".to_string()).unwrap();
        }
        let mut rng = StdRng::from_seed([0; 32]);
        b.iter(|| {
            for _i in 1..(1 << 2) {
                let rand_num = rng.gen::<u8>();
                store.get(format!("{}", rand_num)).unwrap();
            }
        })
    });
    c.bench("kvs_write", bench);
}

criterion_group!(bench, engine_write, engine_read);
criterion_main!(bench);
