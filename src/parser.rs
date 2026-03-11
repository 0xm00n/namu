//! Ternary parser for tree calculus programs.
//!
//! Ternary encoding: 0 = Leaf, 1x = Stem(x), 2xy = Fork(x,y)

use crate::tree::{Trees, TreeIndex};

pub fn parse_ternary(trees: &mut Trees, s: &str) -> Result<TreeIndex, String> {
    let (tree, rest) = parse_ternary_inner(trees, s.as_bytes())?;
    if rest.is_empty() {
        Ok(tree)
    } else {
        Err(format!("trailing input: {} bytes remaining", rest.len()))
    }
}

fn parse_ternary_inner<'a>(trees: &mut Trees, s: &'a [u8]) -> Result<(TreeIndex, &'a [u8]), String> {
    match s.first() {
        Some(b'0') => Ok((trees.leaf(), &s[1..])),
        Some(b'1') => {
            let (child, rest) = parse_ternary_inner(trees, &s[1..])?;
            Ok((trees.stem(child), rest))
        }
        Some(b'2') => {
            let (left, rest) = parse_ternary_inner(trees, &s[1..])?;
            let (right, rest2) = parse_ternary_inner(trees, rest)?;
            Ok((trees.fork(left, right), rest2))
        }
        Some(c) => Err(format!("unexpected byte in ternary: '{}'", *c as char)),
        None => Err("unexpected end of ternary input".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::*;

    #[test]
    fn test_ternary_leaf() {
        let mut trees = Trees::new();
        let l = trees.leaf();
        assert_eq!(parse_ternary(&mut trees, "0").unwrap(), l);
    }

    #[test]
    fn test_ternary_stem() {
        let mut trees = Trees::new();
        let k_val = k(&mut trees);
        assert_eq!(parse_ternary(&mut trees, "10").unwrap(), k_val);
    }

    #[test]
    fn test_ternary_fork() {
        let mut trees = Trees::new();
        let l = trees.leaf();
        let expected = trees.fork(l, l);
        assert_eq!(parse_ternary(&mut trees, "200").unwrap(), expected);
    }

    #[test]
    fn test_ternary_nested() {
        let mut trees = Trees::new();
        let k_val = k(&mut trees);
        let l = trees.leaf();
        let expected = trees.fork(k_val, l);
        assert_eq!(parse_ternary(&mut trees, "2100").unwrap(), expected);
    }

    #[test]
    fn test_ternary_trailing_error() {
        let mut trees = Trees::new();
        assert!(parse_ternary(&mut trees, "00").is_err());
    }

    #[test]
    fn test_ternary_empty_error() {
        let mut trees = Trees::new();
        assert!(parse_ternary(&mut trees, "").is_err());
    }
}
