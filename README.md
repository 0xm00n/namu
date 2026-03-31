# Namu

Namu, 나무, is Korean for "tree".

Namu is a term rewriting engine, specifically a combinator graph reduction engine, for [triage calculus](https://treecalcul.us), a variant of tree calculus where all programs and data are binary trees reduced by a single operator $\Delta$ and five rules.

Combinator graph reduction is a technique for executing programs by repeatedly rewriting parts of a graph according to fixed rules until no more rules apply. The graph nodes are trees (leaf, stem, fork), and a single combinator $\Delta$ defines how they reduce. There are no variables or variable binding. The engine matches subgraphs against the five triage calculus rules, rewrites them, and repeats until the result is in normal form. Because combinators are just trees in triage calculus, programs are values that can be applied to themselves and to each other, enabling intensional introspection and reflection without the encoding layer required in lambda calculus.

Reduction correctness is verified against [Rocq](https://rocq-prover.org/). The proofs in `rocq_proofs/` are based on `terms.v` from Barry Jay's [typed_tree_calculus](https://github.com/barry-jay-personal/typed_tree_calculus), with `extract_triage.v` added to serialize proven terms as ternary-encoded test vectors for namu. `extract_triage.v` normalizes terms and prints their ternary representations, producing 38 certified input-output pairs that namu parses and checks against its own reduction output at test time. Correctness therefore depends on the Rocq kernel and a small serializer (perhaps there's a bug surface area I haven't thought of yet).

## Building and testing

```
cargo build
cargo test
```

## Benchmarks

`benches/` contains two benchmark suites. `eager_ternary_ref_bench` runs the same programs and parameters as the [test suite](https://github.com/lambada-llc/tree-calculus/blob/main/implementation/cpp/test.cpp) in Johannes Bader's tree-calculus repository, and a matching C++ file is included so both can be run on the same machine. `reduction` uses [Criterion.rs](https://github.com/criterion-rs/criterion.rs) for microbenchmarks of individual combinator operations, ternary parsing, and reduction of the Rocq-proven `equal` and `bf` programs.

```
cargo bench --bench eager_ternary_ref_bench
cargo bench --bench reduction
```

To build and run the C++ baseline (`EagerTernaryRef`):

```
git clone https://github.com/lambada-llc/tree-calculus /tmp/tree-calculus
g++ -O3 -std=c++20 -I /tmp/tree-calculus/implementation/cpp benches/eager_ternary_ref_bench.cpp -o eager_ternary_ref_bench
./eager_ternary_ref_bench
```

The [`EagerTernaryRef`](https://github.com/lambada-llc/tree-calculus/blob/main/implementation/cpp/eager-ternary-ref.hpp) evaluator is a simple flat-buffer implementation with no hash consing and no application cache. Namu gets its speed from two things: during reduction, intermediate trees are appended to the arena without hash consing (matching the flat-buffer approach), and an application cache that persists across `apply()` calls memoizes repeated subproblems. The final result of each `apply()` is hash consed so that structural equality via `==` on `TreeIndex` remains O(1). These are representative numbers from one machine (min of 10 iterations, seconds):

| Benchmark | C++ EagerTernaryRef | Namu |
|---|---|---|
| Linear fib(90) | 0.0050 | 0.0046 |
| Recursive fib(24) | 0.437 | 0.019 |
| Alloc and identity (1M) | 0.099 | 0.038 |

The Criterion microbenchmarks reuse a single `Trees` instance across iterations, so the application cache is warm and the numbers reflect amortized cost:

| Benchmark | Time |
|---|---|
| apply(I, 7) | 9 ns |
| NOT false | 7 ns |
| S K K 5 | 85 ns |
| K 42 99 | 199 ns |
| triage on Stem | 117 ns |
| parse\_ternary equal (780 nodes) | 6.6 µs |
| parse\_ternary bf (877 nodes) | 7.2 µs |
| equal(Leaf, Leaf) | 18 ns |
| equal(K, K) | 32 ns |
| equal(equal, equal) | 32 ns |
| bf(I, Leaf) | 23 ns |
| bf(I, 7) via bf | 33 ns |
| divergent bailout (budget 1000) | 2.9 µs |

## Acknowledgements
- Barry Jay created tree calculus
- Johannes Bader suggested and created triage calculus
- Barry Jay formalized triage calculus in "Typed Program Analysis without Encodings"
- The rust implementation optimizations are based on [Philogy's tree-calculus-rs](https://github.com/Philogy/tree-calculus-rs).
