//! Core tree calculus types and reduction rules (arena-based).
//!
//! Flat arena with hash consing and application cache, 
//! Every tree is an index into the arena.

use std::num::NonZero;
use ahash::AHashMap;

// Index into the arena. NonZero allows Option<TreeIndex> to be 4 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TreeIndex {
    idx: NonZero<u32>,
}

impl TreeIndex {
    fn new(raw: u32) -> Self {
        TreeIndex { idx: NonZero::new(raw).expect("TreeIndex cannot be zero") }
    }

    fn raw(self) -> u32 {
        self.idx.get()
    }
}

// Packed entry: 8 bytes. None/None = Leaf, Some/None = Stem, Some/Some = Fork.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TreeEntry {
    left: Option<TreeIndex>,
    right: Option<TreeIndex>,
}

impl TreeEntry {
    fn leaf() -> Self { TreeEntry { left: None, right: None } }
    fn stem(inner: TreeIndex) -> Self { TreeEntry { left: Some(inner), right: None } }
    fn fork(left: TreeIndex, right: TreeIndex) -> Self { TreeEntry { left: Some(left), right: Some(right) } }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tree {
    Leaf,
    Stem(TreeIndex),
    Fork(TreeIndex, TreeIndex),
}

const MAX_TREE_IDX_TO_BE_CACHED: u32 = 50_000;

enum EvalFrame {
    // S-rule: computed (x b), now need (y b), then apply results
    SRight { y: TreeIndex, b: TreeIndex },
    // S-rule: have left result, need to apply to right result
    SApply { left_result: TreeIndex },
    // Fork-triage: computed (z u), now apply result to v
    ForkV { v: TreeIndex },
    // Cache: when this frame unwinds, cache (a, b) -> result
    Cache { a: TreeIndex, b: TreeIndex },
}

pub struct Trees {
    entries: Vec<TreeEntry>,
    indexed_trees: AHashMap<TreeEntry, TreeIndex>,
    cached_applications: AHashMap<(TreeIndex, TreeIndex), TreeIndex>,
    eval_stack: Vec<EvalFrame>,
}

impl Trees {
    pub fn new() -> Self {
        let mut t = Trees {
            entries: Vec::with_capacity(4096),
            indexed_trees: AHashMap::with_capacity(4096),
            cached_applications: AHashMap::with_capacity(1024),
            eval_stack: Vec::with_capacity(256),
        };
        t.entries.push(TreeEntry::leaf()); // index 0: unused (NonZero)
        let leaf_entry = TreeEntry::leaf();
        let leaf_idx = TreeIndex::new(1);
        t.entries.push(leaf_entry);        // index 1: Leaf
        t.indexed_trees.insert(leaf_entry, leaf_idx);
        t.stem(leaf_idx);                  // index 2: K = Stem(Leaf)
        t.fork(leaf_idx, leaf_idx);        // index 3: Fork(Leaf, Leaf)
        t
    }

    // Constructors  ------------------------------------------------------------

    pub fn leaf(&mut self) -> TreeIndex {
        TreeIndex::new(1)
    }

    pub fn stem(&mut self, inner: TreeIndex) -> TreeIndex {
        self.insert(TreeEntry::stem(inner))
    }

    pub fn fork(&mut self, left: TreeIndex, right: TreeIndex) -> TreeIndex {
        self.insert(TreeEntry::fork(left, right))
    }

    fn insert(&mut self, entry: TreeEntry) -> TreeIndex {
        if let Some(&idx) = self.indexed_trees.get(&entry) {
            return idx;
        }
        let raw = self.entries.len() as u32;
        let idx = TreeIndex::new(raw);
        self.entries.push(entry);
        self.indexed_trees.insert(entry, idx);
        idx
    }

    #[inline(always)]
    fn insert_raw(&mut self, entry: TreeEntry) -> TreeIndex {
        let raw = self.entries.len() as u32;
        let idx = TreeIndex::new(raw);
        self.entries.push(entry);
        idx
    }

    // Lookup -------------------------------------------------------------------

    pub fn index(&self, idx: TreeIndex) -> Tree {
        let e = &self.entries[idx.raw() as usize];
        match (e.left, e.right) {
            (None, None) => Tree::Leaf,
            (Some(inner), None) => Tree::Stem(inner),
            (Some(left), Some(right)) => Tree::Fork(left, right),
            (None, Some(_)) => unreachable!("invalid tree entry"),
        }
    }

    // Reduction ----------------------------------------------------------------

    pub fn apply(&mut self, f: TreeIndex, arg: TreeIndex) -> TreeIndex {
        self.apply_inner(f, arg)
    }

    fn should_cache(&self, a: TreeIndex, b: TreeIndex) -> bool {
        a.raw() <= MAX_TREE_IDX_TO_BE_CACHED && b.raw() <= MAX_TREE_IDX_TO_BE_CACHED
    }

    #[inline(always)]
    fn check_cache(&mut self, f: TreeIndex, arg: TreeIndex) -> Option<TreeIndex> {
        if self.should_cache(f, arg) {
            if let Some(&cached) = self.cached_applications.get(&(f, arg)) {
                return Some(cached);
            }
            self.eval_stack.push(EvalFrame::Cache { a: f, b: arg });
        }
        None
    }

    fn apply_inner(&mut self, mut f: TreeIndex, mut arg: TreeIndex) -> TreeIndex {
        debug_assert!(self.eval_stack.is_empty());

        loop {
            let result = 'reduce: loop {
                match self.index(f) {
                    Tree::Leaf => break 'reduce self.insert_raw(TreeEntry::stem(arg)),
                    Tree::Stem(a) => break 'reduce self.insert_raw(TreeEntry::fork(a, arg)),
                    Tree::Fork(left, right) => match self.index(left) {
                        Tree::Leaf => break 'reduce right,

                        // S-rule: △(△x)y z -> (xz)(yz)
                        Tree::Stem(x) => {
                            if let Some(cached) = self.check_cache(f, arg) {
                                break 'reduce cached;
                            }
                            self.eval_stack.push(EvalFrame::SRight { y: right, b: arg });
                            f = x;
                            if let Some(cached) = self.check_cache(f, arg) {
                                break 'reduce cached;
                            }
                            continue 'reduce;
                        }

                        // Triage rules: △(△wx)y z
                        Tree::Fork(w, x) => match self.index(arg) {
                            Tree::Leaf => break 'reduce w,
                            // Stem-triage: △(△wx)y (△u) -> xu
                            Tree::Stem(u) => {
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                f = x;
                                arg = u;
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                continue 'reduce;
                            }
                            // Fork-triage: △(△wx)y (△uv) -> yuv
                            Tree::Fork(u, v) => {
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                self.eval_stack.push(EvalFrame::ForkV { v });
                                f = right;
                                arg = u;
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                continue 'reduce;
                            }
                        },
                    },
                }
            };

            match self.unwind(result) {
                Some((nf, narg)) => { f = nf; arg = narg; }
                None => {
                    let entry = self.entries[result.raw() as usize];
                    return self.insert(entry);
                }
            }
        }
    }

    fn unwind(&mut self, result: TreeIndex) -> Option<(TreeIndex, TreeIndex)> {
        loop {
            match self.eval_stack.pop() {
                None => return None,
                Some(EvalFrame::Cache { a, b }) => {
                    self.cached_applications.insert((a, b), result);
                }
                Some(EvalFrame::SRight { y, b }) => {
                    self.eval_stack.push(EvalFrame::SApply { left_result: result });
                    return Some((y, b));
                }
                Some(EvalFrame::SApply { left_result }) => {
                    return Some((left_result, result));
                }
                Some(EvalFrame::ForkV { v }) => {
                    return Some((result, v));
                }
            }
        }
    }

    pub fn apply_bounded(
        &mut self,
        f: TreeIndex,
        arg: TreeIndex,
        steps: &mut usize,
        max_steps: usize,
    ) -> Result<TreeIndex, TreeIndex> {
        self.apply_bounded_inner(f, arg, steps, max_steps)
    }

    fn apply_bounded_inner(
        &mut self,
        mut f: TreeIndex,
        mut arg: TreeIndex,
        steps: &mut usize,
        max_steps: usize,
    ) -> Result<TreeIndex, TreeIndex> {
        debug_assert!(self.eval_stack.is_empty());

        macro_rules! check_budget {
            () => {
                *steps += 1;
                if *steps >= max_steps {
                    self.eval_stack.clear();
                    return Err(f);
                }
            };
        }

        loop {
            check_budget!();

            let result = 'reduce: loop {
                match self.index(f) {
                    Tree::Leaf => break 'reduce self.insert_raw(TreeEntry::stem(arg)),
                    Tree::Stem(a) => break 'reduce self.insert_raw(TreeEntry::fork(a, arg)),
                    Tree::Fork(left, right) => match self.index(left) {
                        Tree::Leaf => break 'reduce right,

                        Tree::Stem(x) => {
                            if let Some(cached) = self.check_cache(f, arg) {
                                break 'reduce cached;
                            }
                            self.eval_stack.push(EvalFrame::SRight { y: right, b: arg });
                            f = x;
                            check_budget!();
                            if let Some(cached) = self.check_cache(f, arg) {
                                break 'reduce cached;
                            }
                            continue 'reduce;
                        }

                        Tree::Fork(w, x) => match self.index(arg) {
                            Tree::Leaf => break 'reduce w,
                            Tree::Stem(u) => {
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                f = x;
                                arg = u;
                                check_budget!();
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                continue 'reduce;
                            }
                            Tree::Fork(u, v) => {
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                self.eval_stack.push(EvalFrame::ForkV { v });
                                f = right;
                                arg = u;
                                check_budget!();
                                if let Some(cached) = self.check_cache(f, arg) {
                                    break 'reduce cached;
                                }
                                continue 'reduce;
                            }
                        },
                    },
                }
            };

            match self.unwind(result) {
                Some((nf, narg)) => { f = nf; arg = narg; }
                None => {
                    let entry = self.entries[result.raw() as usize];
                    return Ok(self.insert(entry));
                }
            }
        }
    }

    // Structure queries --------------------------------------------------------

    pub fn size(&self, idx: TreeIndex) -> usize {
        match self.index(idx) {
            Tree::Leaf => 1,
            Tree::Stem(inner) => 1 + self.size(inner),
            Tree::Fork(left, right) => 1 + self.size(left) + self.size(right),
        }
    }

    pub fn depth(&self, idx: TreeIndex) -> usize {
        match self.index(idx) {
            Tree::Leaf => 0,
            Tree::Stem(inner) => 1 + self.depth(inner),
            Tree::Fork(left, right) => 1 + self.depth(left).max(self.depth(right)),
        }
    }

    // Display ------------------------------------------------------------------

    pub fn display(&self, idx: TreeIndex) -> String {
        match self.index(idx) {
            Tree::Leaf => "t".to_string(),
            Tree::Stem(inner) => format!("(t {})", self.display(inner)),
            Tree::Fork(left, right) => format!("(t {} {})", self.display(left), self.display(right)),
        }
    }

    pub fn debug_display(&self, idx: TreeIndex) -> String {
        match self.index(idx) {
            Tree::Leaf => "Leaf".to_string(),
            Tree::Stem(inner) => format!("Stem({})", self.debug_display(inner)),
            Tree::Fork(left, right) => format!("Fork({}, {})", self.debug_display(left), self.debug_display(right)),
        }
    }

    // Arena stats --------------------------------------------------------------

    pub fn arena_size(&self) -> usize {
        self.entries.len()
    }
}
