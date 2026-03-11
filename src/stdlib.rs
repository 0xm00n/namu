//! Standard library of tree calculus combinators and data encodings.

use crate::tree::{Trees, TreeIndex};

// SKI combinators ----------------------------------------------------------------

pub fn k(trees: &mut Trees) -> TreeIndex {
    let l = trees.leaf();
    trees.stem(l)
}

pub fn s(trees: &mut Trees) -> TreeIndex {
    let l = trees.leaf();
    let ll = trees.fork(l, l);
    let s_left = trees.stem(ll);
    trees.fork(s_left, l)
}

pub fn i(trees: &mut Trees) -> TreeIndex {
    let k_val = k(trees);
    let sk = trees.stem(k_val);
    trees.fork(sk, k_val)
}

// Booleans ----------------------------------------------------------------

pub fn tree_false(trees: &mut Trees) -> TreeIndex {
    trees.leaf()
}

pub fn tree_true(trees: &mut Trees) -> TreeIndex {
    let l = trees.leaf();
    trees.stem(l)
}

pub fn tree_not(trees: &mut Trees) -> TreeIndex {
    let t = tree_true(trees);
    let f = tree_false(trees);
    let l = trees.leaf();
    let inner = trees.fork(l, f);
    let left = trees.fork(t, inner);
    trees.fork(left, l)
}

// Triage ----------------------------------------------------------------

pub fn triage(trees: &mut Trees, leaf_case: TreeIndex, stem_case: TreeIndex, fork_case: TreeIndex) -> TreeIndex {
    let left = trees.fork(leaf_case, stem_case);
    trees.fork(left, fork_case)
}

// Number encoding ----------------------------------------------------------------

pub fn to_number(trees: &mut Trees, n: usize) -> TreeIndex {
    if n == 0 {
        return trees.leaf();
    }
    let l = trees.leaf();
    let bit = if n & 1 == 1 {
        trees.stem(l)
    } else {
        l
    };
    let rest = to_number(trees, n >> 1);
    trees.fork(bit, rest)
}

pub fn of_number(trees: &Trees, idx: TreeIndex) -> Option<usize> {
    use crate::tree::Tree;
    match trees.index(idx) {
        Tree::Leaf => Some(0),
        Tree::Fork(bit, rest) => {
            let b = match trees.index(bit) {
                Tree::Leaf => 0,
                Tree::Stem(inner) => {
                    if trees.index(inner) == Tree::Leaf { 1 } else { return None; }
                }
                _ => return None,
            };
            of_number(trees, rest).map(|r| b + (r << 1))
        }
        _ => None,
    }
}

// List encoding ----------------------------------------------------------------

pub fn to_list(trees: &mut Trees, items: &[TreeIndex]) -> TreeIndex {
    let l = trees.leaf();
    items.iter().rev().fold(l, |acc, &item| {
        trees.fork(item, acc)
    })
}

pub fn of_list(trees: &Trees, idx: TreeIndex) -> Option<Vec<TreeIndex>> {
    use crate::tree::Tree;
    let mut res = Vec::new();
    let mut curr = idx;
    loop {
        match trees.index(curr) {
            Tree::Leaf => return Some(res),
            Tree::Fork(head, tail) => {
                res.push(head);
                curr = tail;
            }
            _ => return None,
        }
    }
}

// String encoding ----------------------------------------------------------------

pub fn to_string_tree(trees: &mut Trees, s: &str) -> TreeIndex {
    let chars: Vec<TreeIndex> = s.chars().map(|c| to_number(trees, c as usize)).collect();
    to_list(trees, &chars)
}

pub fn of_string_tree(trees: &Trees, idx: TreeIndex) -> Option<String> {
    of_list(trees, idx)?
        .iter()
        .map(|&item| {
            of_number(trees, item)
                .and_then(|n| char::from_u32(n as u32))
        })
        .collect()
}

// Peano encoding ----------------------------------------------------------------

pub fn peano(trees: &mut Trees, n: usize) -> TreeIndex {
    if n == 0 {
        trees.leaf()
    } else {
        let inner = peano(trees, n - 1);
        trees.stem(inner)
    }
}

// Tests ----------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_k_combinator() {
        let mut trees = Trees::new();
        let a = to_number(&mut trees, 42);
        let b = to_number(&mut trees, 99);
        let k_val = k(&mut trees);
        let ka = trees.apply(k_val, a);
        let res = trees.apply(ka, b);
        assert_eq!(of_number(&trees, res), Some(42));
    }

    #[test]
    fn test_i_combinator() {
        let mut trees = Trees::new();
        let a = to_number(&mut trees, 7);
        let i_val = i(&mut trees);
        let res = trees.apply(i_val, a);
        assert_eq!(of_number(&trees, res), Some(7));
    }

    #[test]
    fn test_skk_is_identity() {
        let mut trees = Trees::new();
        let x = to_number(&mut trees, 5);
        let s_val = s(&mut trees);
        let k_val = k(&mut trees);
        let sk = trees.apply(s_val, k_val);
        let skk = trees.apply(sk, k_val);
        let res = trees.apply(skk, x);
        assert_eq!(of_number(&trees, res), Some(5));
    }

    #[test]
    fn test_not() {
        let mut trees = Trees::new();
        let not = tree_not(&mut trees);
        let f = tree_false(&mut trees);
        let t = tree_true(&mut trees);

        let nf = trees.apply(not, f);
        assert_eq!(nf, t);

        let nt = trees.apply(not, t);
        assert_eq!(nt, f);
    }

    #[test]
    fn test_number_roundtrip() {
        let mut trees = Trees::new();
        for n in 0..100 {
            let encoded = to_number(&mut trees, n);
            assert_eq!(of_number(&trees, encoded), Some(n), "Failed for {}", n);
        }
    }

    #[test]
    fn test_list_roundtrip() {
        let mut trees = Trees::new();
        let items = vec![
            to_number(&mut trees, 1),
            to_number(&mut trees, 2),
            to_number(&mut trees, 3),
        ];
        let encoded = to_list(&mut trees, &items);
        let decoded = of_list(&trees, encoded).unwrap();

        assert_eq!(decoded.len(), 3);
        assert_eq!(of_number(&trees, decoded[0]), Some(1));
        assert_eq!(of_number(&trees, decoded[1]), Some(2));
        assert_eq!(of_number(&trees, decoded[2]), Some(3));
    }

    #[test]
    fn test_string_roundtrip() {
        let mut trees = Trees::new();
        let s = "hello world!";
        let encoded = to_string_tree(&mut trees, s);
        assert_eq!(of_string_tree(&trees, encoded), Some(s.to_string()));
    }

    #[test]
    fn test_triage_leaf() {
        let mut trees = Trees::new();
        let i_val = i(&mut trees);
        let n1 = to_number(&mut trees, 1);
        let t = triage(&mut trees, n1, i_val, i_val);
        let l = trees.leaf();
        let res = trees.apply(t, l);
        assert_eq!(of_number(&trees, res), Some(1));
    }

    #[test]
    fn test_triage_stem() {
        let mut trees = Trees::new();
        let i_val = i(&mut trees);
        let n0 = to_number(&mut trees, 0);
        let t = triage(&mut trees, n0, i_val, i_val);
        let inner = to_number(&mut trees, 42);
        let stem = trees.stem(inner);
        let res = trees.apply(t, stem);
        assert_eq!(of_number(&trees, res), Some(42));
    }
}
