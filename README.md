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

Benchmarks use [Criterion.rs](https://github.com/criterion-rs/criterion.rs) and cover combinator operations, ternary parsing, and reduction of large tree programs:

```
cargo bench
```

## Acknowledgements
- Barry Jay created tree calculus
- Johannes Bader suggested and created triage calculus
- Barry Jay formalized triage calculus in "Typed Program Analysis without Encodings"
- The rust implementation optimizations are based on [Philogy's tree-calculus-rs](https://github.com/Philogy/tree-calculus-rs).
