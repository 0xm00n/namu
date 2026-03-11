use crate::parser;
use crate::tree::{Trees, TreeIndex};

const TRIAGE_VECTORS: &str = include_str!("../triage_vectors.tsv");

fn parse_and_apply(trees: &mut Trees, s: &str) -> TreeIndex {
    let parts: Vec<&str> = s.split_whitespace().collect();
    let indices: Vec<TreeIndex> = parts.iter()
        .map(|p| parser::parse_ternary(trees, p).unwrap_or_else(|e| panic!("bad ternary '{}': {}", p, e)))
        .collect();
    indices.into_iter().reduce(|acc, t| trees.apply(acc, t))
        .expect("empty expression")
}

#[test]
fn rocq_triage_vectors() {
    let mut trees = Trees::new();
    let mut passed = 0;
    let mut failed = Vec::new();

    for line in TRIAGE_VECTORS.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains(" ||| ") { continue; }

        let parts: Vec<&str> = line.split(" ||| ").collect();
        assert_eq!(parts.len(), 3, "malformed line: {}", line);

        let (name, input, expected) = (parts[0], parts[1], parts[2]);
        let got = parse_and_apply(&mut trees, input);
        let want = parse_and_apply(&mut trees, expected);

        if got == want {
            passed += 1;
        } else {
            failed.push(format!("{}: got {} expected {}", name, trees.display(got), trees.display(want)));
        }
    }

    if !failed.is_empty() {
        panic!(
            "{} passed, {} failed:\n{}",
            passed, failed.len(), failed.join("\n")
        );
    }
}
