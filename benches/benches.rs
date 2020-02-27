#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion, ParameterizedBenchmark};
use kvs::{KVEngine, KVPair, KVStore, SledKVEngine};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::{thread_rng, Rng};
use std::convert::TryInto;
use std::iter;
use tempfile::TempDir;

const NUM_ENTRY: u32 = 1000;
const LEN_ENTRY: u32 = 100000;

fn engine_write(c: &mut Criterion) {
    let bench = ParameterizedBenchmark::new(
        "kvs_set",
        |b, _| {
            b.iter_batched(
                || {
                    let temp_dir = TempDir::new().unwrap();
                    (KVStore::open(temp_dir.path()).unwrap(), temp_dir)
                },
                |(store, _temp_dir)| {
                    // for i in 1..(1 << 12) {
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
            |(store, _temp_dir)| {
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
        move |b, _| {
            let temp_dir = TempDir::new().unwrap();
            let store = KVStore::open(temp_dir.path()).unwrap();
            read_setup_and_bench(b, store);
        },
        iter::once(()),
    )
    .with_function("sled_read", |b, _| {
        let temp_dir = TempDir::new().unwrap();
        let store = SledKVEngine::open(temp_dir.path()).unwrap();
        read_setup_and_bench(b, store);
    });
    c.bench("kvs_write", bench);
}

fn read_setup_and_bench<E: KVEngine>(b: &mut criterion::Bencher<'_>, engine: E) {
    let mut vec_pair = generate_pair();

    vec_pair.iter_mut().for_each(|pair| {
        engine
            .set(pair.key.to_owned(), pair.val.to_owned())
            .unwrap();
    });

    let mut rng = thread_rng();
    let mut vec_pair_idx_shuf: Vec<usize> = (1..vec_pair.len()).collect();
    vec_pair_idx_shuf.shuffle(&mut rng);
    b.iter(move || {
        vec_pair_idx_shuf.to_owned().into_iter().for_each(|idx| {
            let key = &vec_pair[idx].key;
            engine.get(key.to_owned()).unwrap();
        });
    })
}

fn generate_pair() -> Vec<KVPair> {
    let mut rng_len = StdRng::from_seed([0; 32]);
    let rng = thread_rng();
    let mut vec_pair = Vec::<KVPair>::new();

    for _i in 1..NUM_ENTRY {
        let lenk: usize = rng_len.gen_range(0, LEN_ENTRY).try_into().unwrap();
        let k: String = rng.sample_iter(Alphanumeric).take(lenk).collect();

        let lenv: usize = rng_len.gen_range(0, LEN_ENTRY).try_into().unwrap();
        let v: String = rng.sample_iter(Alphanumeric).take(lenv).collect();

        vec_pair.push(KVPair { key: k, val: v });
    }

    vec_pair
}

criterion_group!(bench, engine_write, engine_read);
criterion_main!(bench);
