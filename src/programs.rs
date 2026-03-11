//! Core library programs as tree calculus values.
//!
//! `equal` (structural equality), `bf` (branch-first self-evaluator), and
//! `mirror` (self-recognition) are loaded from Rocq-proven ternary representations.

use crate::tree::{Trees, TreeIndex};
use crate::parser::parse_ternary;

const EQUAL_TERNARY: &str = "212121202120112110102121200212002120112002120112002121200212002120102120021200212120021200212010211010212010211010202120102110102020211010202120112110102121200212002120112002120112002121200212002120102120021200212120021200212010211010212010211010202120102110102020211010202120112220221020202110102020202110102121200212002120112002120112012021101021201121101021201021101020202020202110102120112011201220211010202021101021212002120021201120021201120021201120112002120112011200212011201120112002120112011201120021212002120021201021200212002120112002120112002120112011200212011201120021201120112010212120021200212011200212011200212011201021201121101021201021101020202110102021201120102121200212002120112002120112002120112010212011211010212010211010202021101020202020202021101010211010";

const BF_TERNARY: &str = "2121212021201121101021212002120021201120021201120021212002120021201021200212002121200212002120102110102120102110102021201021101020202110102021201121101021212002120021201120021201120021212002120021201021200212002121200212002120102110102120102110102021201021101020202110102021201200212120021201102121200212002120112002120112002120112010212011200212011200212011201120212011212022212110102002120112110102120102120021101021201120112110102120112010212120021200212010212002110102021101021101021201021101021212002120021201021200212002120102110102121200212002120102110102021101021201021212002120021201021101020211010212120021200212011200212011200212011201120021201120112002120112011201021201120112002120112120021200212010212002110102120102121200212002120102110102021101021201021201021212002120021201021212002120021201021200212002120102110102121200212002120102110102021101020211010211010";

const MIRROR_TERNARY: &str = "2121212021201121101021212002120021201120021201120021212002120021201021200212002121200212002120102110102120102110102021201021101020202110102021201121101021212002120021201120021201120021212002120021201021200212002121200212002120102110102120102110102021201021101020202110102021201121212120212011211010212120021200212011200212011200212120021200212010212002120021212002120021201021101021201021101020212010211010202021101020212011211010212120021200212011200212011200212120021200212010212002120021212002120021201021101021201021101020212010211010202021101020212011222022102020211010202020211010212120021200212011200212011201202110102120112110102120102110102020202020211010212011201120122021101020202110102121200212002120112002120112002120112011200212011201120021201120112011200212011201120112002121200212002120102120021200212011200212011200212011201120021201120112002120112011201021212002120021201120021201120021201120102120112110102120102110102020211010202120112010212120021200212011200212011200212011201021201121101021201021101020202110102020202020202110101021101010211010";

pub fn equal(trees: &mut Trees) -> TreeIndex {
    parse_ternary(trees, EQUAL_TERNARY)
        .expect("EQUAL_TERNARY is a Rocq-proven constant; parse must succeed")
}

pub fn bf(trees: &mut Trees) -> TreeIndex {
    parse_ternary(trees, BF_TERNARY)
        .expect("BF_TERNARY is a Rocq-proven constant; parse must succeed")
}

pub fn mirror(trees: &mut Trees) -> TreeIndex {
    parse_ternary(trees, MIRROR_TERNARY)
        .expect("MIRROR_TERNARY is a Rocq-proven constant; parse must succeed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::*;

    #[test]
    fn test_equal_node_count() {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        assert_eq!(trees.size(eq), 780, "must match Rocq's term_size_equal");
    }

    #[test]
    fn test_bf_node_count() {
        let mut trees = Trees::new();
        let b = bf(&mut trees);
        assert_eq!(trees.size(b), 877, "must match Rocq's term_size_bf");
    }

    #[test]
    fn test_equal_leaf_leaf() {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let l = trees.leaf();
        let k_val = k(&mut trees);
        let r1 = trees.apply(eq, l);
        let result = trees.apply(r1, l);
        assert_eq!(result, k_val, "equal(△,△) should be K");
    }

    #[test]
    fn test_equal_k_k() {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let k_val = k(&mut trees);
        let r1 = trees.apply(eq, k_val);
        let result = trees.apply(r1, k_val);
        assert_eq!(result, k_val, "equal(K,K) should be K");
    }

    #[test]
    fn test_equal_leaf_k_is_not_true() {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let l = trees.leaf();
        let k_val = k(&mut trees);
        let r1 = trees.apply(eq, l);
        let result = trees.apply(r1, k_val);
        assert_ne!(result, k_val, "equal(△,K) should not be K");
    }

    #[test]
    fn test_equal_reflexive() {
        let mut trees = Trees::new();
        let eq = equal(&mut trees);
        let r1 = trees.apply(eq, eq);
        let result = trees.apply(r1, eq);
        let k_val = k(&mut trees);
        assert_eq!(result, k_val, "equal should be equal to itself");
    }

    #[test]
    fn test_bf_leaf() {
        let mut trees = Trees::new();
        let b = bf(&mut trees);
        let l = trees.leaf();
        let p = k(&mut trees);
        let r1 = trees.apply(b, l);
        let result = trees.apply(r1, p);
        let expected = trees.stem(p);
        assert_eq!(result, expected, "bf(△,p) should be Stem(p)");
    }

    #[test]
    fn test_bf_k_rule() {
        let mut trees = Trees::new();
        let b = bf(&mut trees);
        let l = trees.leaf();
        let y = to_number(&mut trees, 42);
        let k_with_y = trees.fork(l, y);
        let r1 = trees.apply(b, k_with_y);
        let result = trees.apply(r1, l);
        assert_eq!(result, y, "bf(△△y, z) should be y (K-rule)");
    }

    #[test]
    fn test_bf_identity() {
        let mut trees = Trees::new();
        let b = bf(&mut trees);
        let i_val = i(&mut trees);
        let x = to_number(&mut trees, 7);
        let r1 = trees.apply(b, i_val);
        let result = trees.apply(r1, x);
        assert_eq!(result, x, "bf(I, x) should be x");
    }

    fn ki(trees: &mut Trees) -> TreeIndex {
        let l = trees.leaf();
        let i_val = i(trees);
        trees.fork(l, i_val)
    }

    #[test]
    fn test_mirror_node_count() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        assert_eq!(trees.size(m), 1066, "must match Rocq's term_size mirror");
    }

    #[test]
    fn test_mirror_self() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let k_val = k(&mut trees);
        let result = trees.apply(m, m);
        assert_eq!(result, k_val, "mirror must recognize itself");
    }

    #[test]
    fn test_mirror_leaf() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let l = trees.leaf();
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, l);
        assert_eq!(result, ki_val, "mirror must reject leaf");
    }

    #[test]
    fn test_mirror_k() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let k_val = k(&mut trees);
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, k_val);
        assert_eq!(result, ki_val, "mirror must reject K");
    }

    #[test]
    fn test_mirror_equal() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let eq = equal(&mut trees);
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, eq);
        assert_eq!(result, ki_val, "mirror must reject equal");
    }

    #[test]
    fn test_mirror_bf() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let b = bf(&mut trees);
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, b);
        assert_eq!(result, ki_val, "mirror must reject bf");
    }

    #[test]
    fn test_mirror_i() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let i_val = i(&mut trees);
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, i_val);
        assert_eq!(result, ki_val, "mirror must reject I");
    }

    #[test]
    fn test_mirror_stem_leaf() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let l = trees.leaf();
        let sl = trees.stem(l);
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, sl);
        assert_eq!(result, ki_val, "mirror must reject Stem(Leaf)");
    }

    #[test]
    fn test_mirror_fork_leaf_leaf() {
        let mut trees = Trees::new();
        let m = mirror(&mut trees);
        let l = trees.leaf();
        let fll = trees.fork(l, l);
        let ki_val = ki(&mut trees);
        let result = trees.apply(m, fll);
        assert_eq!(result, ki_val, "mirror must reject Fork(Leaf,Leaf)");
    }
}
