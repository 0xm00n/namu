use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

use namu::tree::Trees;
use namu::stdlib::*;
use namu::parser::parse_ternary;
use namu::programs::{equal, bf};

const EQUAL_TERNARY: &str = "212121202120112110102121200212002120112002120112002121200212002120102120021200212120021200212010211010212010211010202120102110102020211010202120112110102121200212002120112002120112002121200212002120102120021200212120021200212010211010212010211010202120102110102020211010202120112220221020202110102020202110102121200212002120112002120112012021101021201121101021201021101020202020202110102120112011201220211010202021101021212002120021201120021201120021201120112002120112011200212011201120112002120112011201120021212002120021201021200212002120112002120112002120112011200212011201120021201120112010212120021200212011200212011200212011201021201121101021201021101020202110102021201120102121200212002120112002120112002120112010212011211010212010211010202021101020202020202021101010211010";

const BF_TERNARY: &str = "2121212021201121101021212002120021201120021201120021212002120021201021200212002121200212002120102110102120102110102021201021101020202110102021201121101021212002120021201120021201120021212002120021201021200212002121200212002120102110102120102110102021201021101020202110102021201200212120021201102121200212002120112002120112002120112010212011200212011200212011201120212011212022212110102002120112110102120102120021101021201120112110102120112010212120021200212010212002110102021101021101021201021101021212002120021201021200212002120102110102121200212002120102110102021101021201021212002120021201021101020211010212120021200212011200212011200212011201120021201120112002120112011201021201120112002120112120021200212010212002110102120102121200212002120102110102021101021201021201021212002120021201021212002120021201021200212002120102110102121200212002120102110102021101020211010211010";

fn bench_k(c: &mut Criterion) {
    c.bench_function("K 42 99", |b| {
        let mut trees = Trees::new();
        let k_val = k(&mut trees);
        b.iter(|| {
            let n42 = to_number(&mut trees, 42);
            let n99 = to_number(&mut trees, 99);
            let ka = trees.apply(k_val, black_box(n42));
            trees.apply(ka, black_box(n99))
        })
    });
}

fn bench_skk(c: &mut Criterion) {
    c.bench_function("S K K 5", |b| {
        let mut trees = Trees::new();
        let s_val = s(&mut trees);
        let k_val = k(&mut trees);
        b.iter(|| {
            let sk = trees.apply(s_val, k_val);
            let skk = trees.apply(sk, k_val);
            let n5 = to_number(&mut trees, 5);
            trees.apply(skk, black_box(n5))
        })
    });
}

fn bench_not(c: &mut Criterion) {
    c.bench_function("NOT false", |b| {
        let mut trees = Trees::new();
        let not = tree_not(&mut trees);
        let f = tree_false(&mut trees);
        b.iter(|| {
            trees.apply(not, black_box(f))
        })
    });
}

fn bench_triage(c: &mut Criterion) {
    c.bench_function("triage on Stem", |b| {
        let mut trees = Trees::new();
        let k_val = k(&mut trees);
        let n0 = to_number(&mut trees, 0);
        let n1 = to_number(&mut trees, 1);
        let n2 = to_number(&mut trees, 2);
        let kn1 = trees.apply(k_val, n1);
        let kn2 = trees.apply(k_val, n2);
        let kkn2 = trees.apply(k_val, kn2);
        let type_test = triage(&mut trees, n0, kn1, kkn2);
        b.iter(|| {
            let n99 = to_number(&mut trees, 99);
            let stem99 = trees.stem(n99);
            trees.apply(type_test, black_box(stem99))
        })
    });
}

fn bench_divergent_bailout(c: &mut Criterion) {
    c.bench_function("divergent bailout: budget 1000", |b| {
        let mut trees = Trees::new();
        let s_val = s(&mut trees);
        let i_val = i(&mut trees);
        let si = trees.apply(s_val, i_val);
        let si2 = trees.apply(s_val, i_val);
        let omega = trees.apply(si, si2);
        b.iter(|| {
            let mut steps = 0;
            black_box(trees.apply_bounded(omega, omega, &mut steps, 1000))
        })
    });
}

fn bench_parse_ternary_equal(c: &mut Criterion) {
    c.bench_function("parse_ternary equal (780 nodes)", |b| {
        let mut trees = Trees::new();
        b.iter(|| parse_ternary(&mut trees, black_box(EQUAL_TERNARY)).unwrap())
    });
}

fn bench_parse_ternary_bf(c: &mut Criterion) {
    c.bench_function("parse_ternary bf (877 nodes)", |b| {
        let mut trees = Trees::new();
        b.iter(|| parse_ternary(&mut trees, black_box(BF_TERNARY)).unwrap())
    });
}

fn bench_equal_leaf_leaf(c: &mut Criterion) {
    c.bench_function("equal(Leaf, Leaf)", |b| {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let l = trees.leaf();
        b.iter(|| {
            let r1 = trees.apply(eq, black_box(l));
            trees.apply(r1, black_box(l))
        })
    });
}

fn bench_equal_k_k(c: &mut Criterion) {
    c.bench_function("equal(K, K)", |b| {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let k_val = k(&mut trees);
        b.iter(|| {
            let r1 = trees.apply(eq, black_box(k_val));
            trees.apply(r1, black_box(k_val))
        })
    });
}

fn bench_equal_leaf_k(c: &mut Criterion) {
    c.bench_function("equal(Leaf, K) -> KI", |b| {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let l = trees.leaf();
        let k_val = k(&mut trees);
        b.iter(|| {
            let r1 = trees.apply(eq, black_box(l));
            trees.apply(r1, black_box(k_val))
        })
    });
}

fn bench_equal_reflexive(c: &mut Criterion) {
    c.bench_function("equal(equal, equal)", |b| {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        b.iter(|| {
            let r1 = trees.apply(eq, black_box(eq));
            trees.apply(r1, black_box(eq))
        })
    });
}

fn bench_bf_leaf_k(c: &mut Criterion) {
    c.bench_function("bf(Leaf, K) -> Stem(K)", |b| {
        let mut trees = Trees::new();
        let bf_val = bf(&mut trees);
        let l = trees.leaf();
        let k_val = k(&mut trees);
        b.iter(|| {
            let r1 = trees.apply(bf_val, black_box(l));
            trees.apply(r1, black_box(k_val))
        })
    });
}

fn bench_bf_identity(c: &mut Criterion) {
    c.bench_function("bf(I, Leaf) -> Leaf", |b| {
        let mut trees = Trees::new();
        let bf_val = bf(&mut trees);
        let i_val = i(&mut trees);
        let l = trees.leaf();
        b.iter(|| {
            let r1 = trees.apply(bf_val, black_box(i_val));
            trees.apply(r1, black_box(l))
        })
    });
}

fn bench_bf_k_rule(c: &mut Criterion) {
    c.bench_function("bf(Fork(Leaf,K), Leaf) -> K", |b| {
        let mut trees = Trees::new();
        let bf_val = bf(&mut trees);
        let l = trees.leaf();
        let k_val = k(&mut trees);
        let k_tree = trees.fork(l, k_val);
        b.iter(|| {
            let r1 = trees.apply(bf_val, black_box(k_tree));
            trees.apply(r1, black_box(l))
        })
    });
}

fn bench_direct_apply_i_7(c: &mut Criterion) {
    c.bench_function("direct: apply(I, 7)", |b| {
        let mut trees = Trees::new();
        let i_val = i(&mut trees);
        let n7 = to_number(&mut trees, 7);
        b.iter(|| {
            trees.apply(i_val, black_box(n7))
        })
    });
}

fn bench_bf_apply_i_7(c: &mut Criterion) {
    c.bench_function("bf: apply(apply(bf, I), 7)", |b| {
        let mut trees = Trees::new();
        let bf_val = bf(&mut trees);
        let i_val = i(&mut trees);
        let n7 = to_number(&mut trees, 7);
        b.iter(|| {
            let r1 = trees.apply(bf_val, black_box(i_val));
            trees.apply(r1, black_box(n7))
        })
    });
}

fn bench_number_roundtrip(c: &mut Criterion) {
    c.bench_function("to_number(100) + of_number roundtrip", |b| {
        let mut trees = Trees::new();
        b.iter(|| {
            let encoded = to_number(&mut trees, black_box(100));
            black_box(of_number(&trees, encoded))
        })
    });
}

criterion_group!(
    benches,
    bench_k,
    bench_skk,
    bench_not,
    bench_triage,
    bench_divergent_bailout,
    bench_parse_ternary_equal,
    bench_parse_ternary_bf,
    bench_equal_leaf_leaf,
    bench_equal_k_k,
    bench_equal_leaf_k,
    bench_equal_reflexive,
    bench_bf_leaf_k,
    bench_bf_identity,
    bench_bf_k_rule,
    bench_direct_apply_i_7,
    bench_bf_apply_i_7,
    bench_number_roundtrip,
);
criterion_main!(benches);
