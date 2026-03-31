// Rust translation of the benchmarks from Johannes Bader's tree-calculus test suite.
// https://github.com/lambada-llc/tree-calculus/blob/main/implementation/cpp/test.cpp
//
// The C++ baseline using EagerTernaryRef is in benches/eager_ternary_ref_bench.cpp.
//
// To run:  cargo bench --bench eager_ternary_ref_bench

use std::time::Instant;
use namu::tree::Trees;
use namu::parser::{parse_ternary, to_ternary};
use namu::stdlib::{to_number, of_number, to_string_tree, of_string_tree};

const BENCH_RECURSIVE_FIB: &str = "\
21212021212011212110021100102021202121202120002120112021212120112000202021212\
01121211002110010202120212012210002121202121202121202120002120102120002010212\
02120112120112000101020011201020110212011212011212110021100101020021202120112\
12021202120001021202120112110010212120112121100211001020212021201200212021212\
12011200020202200212011201002001120110212011212011212110021100101010212120212\
02120001021200021202121202120002120102120002010212120212000102021212011212110\
02110010202120212012002120212121201120002020220021201120100200112011021201121\
20112121100211001010202210200202002120112120112121100211001010212120112121100\
21100102021202120120021202121212011200020202200212011201002001120110212011212\
0112121100211001010200";

const BENCH_LINEAR_FIB: &str = "\
21202200102121212011212110021100102021202120122110002120112011201200212120212\
12021200021201021200020102120112021212021200021201021200020102120112011202120\
21200010212011201120212021200010212120212021200010212021200010202120112021212\
01121211002110010202120212012002120212121201120002020220021201120100200112011\
02120112120112121100211001010020212011021212021212021200021201021200020102120\
11202121201121211002110010202120212012220202100002121202120212000102120212120\
21200021201021200020102120112021202120001021201120120011202120112120212021200\
01001020212011212012002222210200202121200221020002110002022212120022102000211\
00202102010001021201121201121211002110010102121201121211002110010202120212011\
21202120002220212120112121100211001020212021201200212011212021202120001021200\
10102120112120112121100211001010021201021201221212011212110021100102021202120\
12002120112120212021200010212002102001021201121201121211002110010100212011201\
02120112000212012211000212021201121202120002120112021212011200020001011201021\
20112120112121100211001010200212011212011212110021100101020221002100";

const BENCH_ALLOC_AND_IDENTITY: &str = "\
21202121100201021212021200021201120212011211001021212021212021200021201021200\
02010212011202121202120002120102120002010212011212021212021200021201021200020\
10212110020212120212120212000212010212000201021212011212110021100102021202120\
21212021200021201021200020102120112012211000212011212021212021200021201021200\
02010212011202120212000102120112021202120001021212021212021212021200021201021\
20002010212021201121201120001010200020211002120112011202121202120001021100212\
120212000212010212000201021201121201121211002110010102001020020212000";

fn test_basic_reduction_rules(trees: &mut Trees) {
    let rule_check = |trees: &mut Trees, rule: &str, expected: &str, a: &str, b: &str| {
        let ta = parse_ternary(trees, a).unwrap();
        let tb = parse_ternary(trees, b).unwrap();
        let result = trees.apply(ta, tb);
        let actual = to_ternary(trees, result);
        if actual != expected {
            panic!(
                "rule {} failed: {} {} --> {} expected but got {}",
                rule, a, b, expected, actual
            );
        }
    };

    let tl = "0";
    let ts = "10";
    let tf = "200";
    let t = [tl, ts, tf];

    for z in &t {
        rule_check(trees, "0a", &format!("1{}", z), "0", z);
    }

    for y in &t {
        for z in &t {
            rule_check(trees, "0b", &format!("2{}{}", y, z), &format!("1{}", y), z);
        }
    }

    for y in &t {
        for z in &t {
            rule_check(trees, "1", y, &format!("20{}", y), z);
        }
    }

    for z in &t {
        rule_check(trees, "2", &format!("2{}1{}", z, z), "2100", z); // x = 0, y = 0
    }

    for yc in &t {
        for z in &t {
            rule_check(trees, "2", &format!("2{}2{}{}", z, yc, z), &format!("2101{}", yc), z); // x = 0, y = 1+yz
        }
    }

    for y in &t {
        for z in &t {
            rule_check(trees, "2", z, &format!("2110{}", y), z); // x = 10
        }
    }

    for w in &t {
        for x in &t {
            for y in &t {
                rule_check(trees, "3a", w, &format!("22{}{}{}", w, x, y), "0");
            }
        }
    }

    for w in &t {
        for y in &t {
            for u in &t {
                rule_check(trees, "3b", &format!("1{}", u), &format!("22{}0{}", w, y), &format!("1{}", u)); // x = 0
            }
        }
    }

    for w in &t {
        for y in &t {
            for u in &t {
                rule_check(trees, "3b", &format!("20{}", u), &format!("22{}10{}", w, y), &format!("1{}", u)); // x = 10
            }
        }
    }

    for w in &t {
        for x in &t {
            for u in &t {
                for v in &t {
                    rule_check(trees, "3c", &format!("2{}{}", u, v), &format!("22{}{}0", w, x), &format!("2{}{}", u, v)); // y = 0
                }
            }
        }
    }

    for w in &t {
        for x in &t {
            for u in &t {
                for v in &t {
                    rule_check(trees, "3c", u, &format!("22{}{}10", w, x), &format!("2{}{}", u, v)); // y = 10
                }
            }
        }
    }
}

fn expected_fib(n: u32) -> i64 {
    let (mut a, mut b): (i64, i64) = (0, 1);
    for _ in 0..=n {
        let t = a + b;
        a = b;
        b = t;
    }
    a
}

fn repeat_measure_sec<F: FnMut()>(mut f: F, iterations: usize) -> Vec<f64> {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        samples.push(start.elapsed().as_secs_f64());
    }
    samples
}

fn print_statistics(title: &str, samples: &[f64]) {
    let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    println!("Benchmark: {} (in seconds)", title);
    println!("  Min: {}", min);
    println!("  Max: {}", max);
    println!("  Average: {}", avg);
    println!("  Median: {}", median);
}

fn main() {
    let name = "Namu";
    println!("Testing {}...", name);

    let mut trees = Trees::new();
    test_basic_reduction_rules(&mut trees);
    println!("  All reduction rules passed.");
    println!("    Stats: {} entries in arena", trees.arena_size());

    let bench_recursive_fib = parse_ternary(&mut trees, BENCH_RECURSIVE_FIB).unwrap();
    let bench_linear_fib = parse_ternary(&mut trees, BENCH_LINEAR_FIB).unwrap();
    let n10 = to_number(&mut trees, 10);
    let result = trees.apply(bench_recursive_fib, n10);
    assert_eq!(of_number(&trees, result), Some(89), "fib misbehavior");
    let n10 = to_number(&mut trees, 10);
    let result = trees.apply(bench_linear_fib, n10);
    assert_eq!(of_number(&trees, result), Some(89), "fib misbehavior");
    println!("  Fib behaves as expected.");
    println!("    Stats: {} entries in arena", trees.arena_size());

    // alloc_and_identity sanity check (from TypeScript suite)
    let prog = parse_ternary(&mut trees, BENCH_ALLOC_AND_IDENTITY).unwrap();
    let n1000 = to_number(&mut trees, 1000);
    let hello = to_string_tree(&mut trees, "hello world");
    let r1 = trees.apply(prog, n1000);
    let result = trees.apply(r1, hello);
    assert_eq!(of_string_tree(&trees, result).unwrap(), "hello world", "alloc_and_identity mismatch");

    let iterations = 10;

    print_statistics(
        &format!("[{}] Setup, should be negligibly fast", name),
        &repeat_measure_sec(|| {
            let mut trees = Trees::new();
            let _ = parse_ternary(&mut trees, BENCH_RECURSIVE_FIB).unwrap();
            let _ = parse_ternary(&mut trees, BENCH_LINEAR_FIB).unwrap();
        }, iterations),
    );

    // C++ uses fib(90) and fib(24) for EagerTernaryRef
    let linear_fib_n: u32 = 90;
    let recursive_fib_n: u32 = 24;
    let expected_linear = expected_fib(linear_fib_n) as usize;
    let expected_recursive = expected_fib(recursive_fib_n) as usize;

    print_statistics(
        &format!("[{}] Linear fib({})", name, linear_fib_n),
        &repeat_measure_sec(|| {
            let mut trees = Trees::new();
            let fib = parse_ternary(&mut trees, BENCH_LINEAR_FIB).unwrap();
            let input = to_number(&mut trees, linear_fib_n as usize);
            let result = trees.apply(fib, input);
            assert_eq!(of_number(&trees, result), Some(expected_linear), "fib misbehavior");
        }, iterations),
    );

    print_statistics(
        &format!("[{}] Recursive fib({})", name, recursive_fib_n),
        &repeat_measure_sec(|| {
            let mut trees = Trees::new();
            let fib = parse_ternary(&mut trees, BENCH_RECURSIVE_FIB).unwrap();
            let input = to_number(&mut trees, recursive_fib_n as usize);
            let result = trees.apply(fib, input);
            assert_eq!(of_number(&trees, result), Some(expected_recursive), "fib misbehavior");
        }, iterations),
    );

    for n in [1000, 1_000_000] {
        print_statistics(
            &format!("[{}] Alloc and identity (n={})", name, n),
            &repeat_measure_sec(|| {
                let mut trees = Trees::new();
                let prog = parse_ternary(&mut trees, BENCH_ALLOC_AND_IDENTITY).unwrap();
                let input = to_number(&mut trees, n);
                let hello = to_string_tree(&mut trees, "hello world");
                let r1 = trees.apply(prog, input);
                let result = trees.apply(r1, hello);
                assert_eq!(of_string_tree(&trees, result).unwrap(), "hello world");
            }, iterations),
        );
    }

    println!();
    println!("All tests passed.");
}
