//! Colour names at three levels, from the bundled lists in `data/`:
//!
//! - basic: 11 basic colour terms (`names_basic.json`)
//! - descriptive: CSS named colours (`names_css.json`)
//! - precise: XKCD colour survey names (`names_xkcd.json`)
//!
//! Lookup: a k-d tree over CIELAB (chroma-compressed, see `tree_point`) finds the K nearest
//! entries by Euclidean distance, then the closest by CIEDE2000 among them wins.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::Rgb;
use crate::convert::Lab;
use crate::delta_e::ciede2000;

/// Candidates refined with CIEDE2000 after the Euclidean k-d tree search. Measured over a
/// 32³ RGB grid against exhaustive search: K = 64 picks a different XKCD name for 0.05% of
/// colours (at most 1.0 ΔE worse) at ~10 µs per lookup; K = 16 differed for 2.2%.
const K: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorName {
    pub name: String,
    pub hex: String,
    /// CIEDE2000 distance from the looked-up colour (0 for search results).
    pub delta_e: f32,
}

#[derive(Debug, Deserialize)]
struct ListFile {
    colors: Vec<RawEntry>,
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    name: String,
    hex: String,
    #[serde(default)]
    alias: bool,
}

#[derive(Debug, Clone)]
struct Entry {
    name: String,
    hex: String,
    lab: Lab,
    alias: bool,
}

/// How [`NameList::nearest`] searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lookup {
    /// k-d tree pre-selection, then CIEDE2000 over the candidates.
    Tree,
    /// CIEDE2000 against every entry. For the basic-term anchors: few anchors cover some terms
    /// (white, black), which Euclidean pre-selection can miss.
    Exhaustive,
}

/// One name list with its lookup index.
#[derive(Debug)]
pub struct NameList {
    entries: Vec<Entry>,
    tree: KdTree,
    lookup: Lookup,
}

impl NameList {
    pub fn from_json(json: &str, lookup: Lookup) -> Result<Self, String> {
        let file: ListFile = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let entries = file
            .colors
            .into_iter()
            .map(|raw| {
                let rgb = Rgb::from_hex(&raw.hex).ok_or_else(|| format!("bad hex {}", raw.hex))?;
                Ok(Entry {
                    name: raw.name,
                    hex: rgb.to_hex(),
                    lab: rgb.to_lab(),
                    alias: raw.alias,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        // Aliases (e.g. "aqua" for "cyan") are searchable but never returned by lookup.
        let points = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.alias)
            .map(|(i, e)| (tree_point(e.lab), i))
            .collect();
        Ok(Self {
            tree: KdTree::build(points),
            entries,
            lookup,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Closest entry by CIEDE2000 (see [`Lookup`]).
    pub fn nearest(&self, lab: Lab) -> ColorName {
        if self.lookup == Lookup::Exhaustive {
            return self.nearest_brute_force(lab);
        }
        let (index, de) = self
            .tree
            .k_nearest(tree_point(lab), K)
            .into_iter()
            .map(|i| (i, ciede2000(lab, self.entries[i].lab)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .expect("name list is empty");
        let e = &self.entries[index];
        ColorName {
            name: e.name.clone(),
            hex: e.hex.clone(),
            delta_e: de as f32,
        }
    }

    /// Every non-alias entry with its Lab value.
    pub fn colors(&self) -> impl Iterator<Item = (ColorName, Lab)> + '_ {
        self.entries.iter().filter(|e| !e.alias).map(|e| {
            (
                ColorName {
                    name: e.name.clone(),
                    hex: e.hex.clone(),
                    delta_e: 0.0,
                },
                e.lab,
            )
        })
    }

    /// The closest entry for each distinct name, best first (exhaustive). For the basic list
    /// this ranks the basic terms, e.g. sienna: orange 7.7, brown 8.2, red …
    pub fn ranked_names(&self, lab: Lab) -> Vec<ColorName> {
        let mut best: Vec<(f64, &Entry)> = Vec::new();
        for e in self.entries.iter().filter(|e| !e.alias) {
            let de = ciede2000(lab, e.lab);
            match best.iter_mut().find(|(_, b)| b.name == e.name) {
                Some(slot) if de < slot.0 => *slot = (de, e),
                Some(_) => {}
                None => best.push((de, e)),
            }
        }
        best.sort_by(|a, b| a.0.total_cmp(&b.0));
        best.into_iter()
            .map(|(de, e)| ColorName {
                name: e.name.clone(),
                hex: e.hex.clone(),
                delta_e: de as f32,
            })
            .collect()
    }

    /// Reference implementation: CIEDE2000 against every entry (used by tests and benchmarks).
    pub fn nearest_brute_force(&self, lab: Lab) -> ColorName {
        let e = self
            .entries
            .iter()
            .filter(|e| !e.alias)
            .min_by(|a, b| ciede2000(lab, a.lab).total_cmp(&ciede2000(lab, b.lab)))
            .expect("name list is empty");
        ColorName {
            name: e.name.clone(),
            hex: e.hex.clone(),
            delta_e: ciede2000(lab, e.lab) as f32,
        }
    }
}

/// The three bundled name lists.
#[derive(Debug)]
pub struct Names {
    pub basic: NameList,
    pub descriptive: NameList,
    pub precise: NameList,
}

impl Names {
    /// Lists compiled into the binary from `data/`, parsed once.
    pub fn bundled() -> &'static Names {
        static NAMES: OnceLock<Names> = OnceLock::new();
        NAMES.get_or_init(|| Names {
            basic: NameList::from_json(
                include_str!("../../../data/names_basic.json"),
                Lookup::Exhaustive,
            )
            .expect("names_basic.json"),
            descriptive: NameList::from_json(
                include_str!("../../../data/names_css.json"),
                Lookup::Tree,
            )
            .expect("names_css.json"),
            precise: NameList::from_json(
                include_str!("../../../data/names_xkcd.json"),
                Lookup::Tree,
            )
            .expect("names_xkcd.json"),
        })
    }

    /// `(basic, descriptive, precise)` names for a colour.
    pub fn name(&self, rgb: Rgb) -> (ColorName, ColorName, ColorName) {
        let lab = rgb.to_lab();
        (
            self.basic.nearest(lab),
            self.descriptive.nearest(lab),
            self.precise.nearest(lab),
        )
    }

    /// Fuzzy name search across all lists ("skyblue", "sky blue" and "Sky-Blue" all match
    /// "sky blue"). Best matches first; each name appears once (descriptive list wins).
    pub fn search(&self, query: &str, limit: usize) -> Vec<ColorName> {
        let q = normalize(query);
        if q.is_empty() {
            return Vec::new();
        }
        let lists = [&self.descriptive, &self.precise, &self.basic];
        let mut hits: Vec<(u32, usize, bool, usize, &Entry)> = Vec::new();
        for (priority, list) in lists.iter().enumerate() {
            for e in &list.entries {
                if let Some(score) = match_score(&q, &normalize(&e.name)) {
                    let len_diff = e.name.len().abs_diff(query.len());
                    hits.push((score, len_diff, e.alias, priority, e));
                }
            }
        }
        // Better score, then closer length, then preferred spelling, then list priority.
        hits.sort_by_key(|h| (h.0, h.1, h.2, h.3));

        let mut seen = std::collections::HashSet::new();
        hits.into_iter()
            .filter(|(.., e)| seen.insert(e.name.clone()))
            .take(limit)
            .map(|(.., e)| ColorName {
                name: e.name.clone(),
                hex: e.hex.clone(),
                delta_e: 0.0,
            })
            .collect()
    }
}

/// k-d tree coordinates: Lab with a*/b* compressed like CIEDE2000's chroma weighting
/// (S_C = 1 + 0.045·C), so Euclidean distance in the tree tracks CIEDE2000 more closely than
/// plain Lab does for saturated colours.
fn tree_point(lab: Lab) -> [f64; 3] {
    let s = 1.0 + 0.045 * lab.a.hypot(lab.b);
    [lab.l, lab.a / s, lab.b / s]
}

/// Lowercase, alphanumerics only, "gray" spelled "grey".
fn normalize(s: &str) -> String {
    s.to_lowercase()
        .replace("gray", "grey")
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Lower is better; `None` means no match.
fn match_score(query: &str, name: &str) -> Option<u32> {
    if query == name {
        Some(0)
    } else if name.starts_with(query) {
        Some(1)
    } else if name.contains(query) {
        Some(2)
    } else {
        let d = levenshtein(query, name);
        let allowed = (query.chars().count() / 4).max(1);
        (d <= allowed).then_some(3 + d as u32)
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

// ---------------------------------------------------------------------------------------------
// k-d tree over 3D points (Lab), K-nearest-neighbour by squared Euclidean distance
// ---------------------------------------------------------------------------------------------

#[derive(Debug)]
struct KdTree {
    nodes: Vec<KdNode>,
    root: Option<usize>,
}

#[derive(Debug)]
struct KdNode {
    point: [f64; 3],
    item: usize,
    axis: usize,
    left: Option<usize>,
    right: Option<usize>,
}

#[derive(PartialEq)]
struct Candidate(f64, usize);

impl Eq for Candidate {}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl KdTree {
    fn build(mut points: Vec<([f64; 3], usize)>) -> Self {
        let mut nodes = Vec::with_capacity(points.len());
        let root = Self::build_rec(&mut points, 0, &mut nodes);
        Self { nodes, root }
    }

    fn build_rec(
        points: &mut [([f64; 3], usize)],
        depth: usize,
        nodes: &mut Vec<KdNode>,
    ) -> Option<usize> {
        if points.is_empty() {
            return None;
        }
        let axis = depth % 3;
        points.sort_by(|a, b| a.0[axis].total_cmp(&b.0[axis]));
        let mid = points.len() / 2;
        let (point, item) = points[mid];
        let index = nodes.len();
        nodes.push(KdNode {
            point,
            item,
            axis,
            left: None,
            right: None,
        });
        let (left, rest) = points.split_at_mut(mid);
        let left = Self::build_rec(left, depth + 1, nodes);
        let right = Self::build_rec(&mut rest[1..], depth + 1, nodes);
        nodes[index].left = left;
        nodes[index].right = right;
        Some(index)
    }

    /// Items of the `k` nearest points, unordered.
    fn k_nearest(&self, target: [f64; 3], k: usize) -> Vec<usize> {
        let mut heap = BinaryHeap::with_capacity(k + 1);
        if let Some(root) = self.root {
            self.search(root, target, k, &mut heap);
        }
        heap.into_iter().map(|Candidate(_, item)| item).collect()
    }

    fn search(&self, node: usize, target: [f64; 3], k: usize, heap: &mut BinaryHeap<Candidate>) {
        let n = &self.nodes[node];
        let d2: f64 = (0..3).map(|i| (n.point[i] - target[i]).powi(2)).sum();
        if heap.len() < k {
            heap.push(Candidate(d2, n.item));
        } else if d2 < heap.peek().unwrap().0 {
            heap.pop();
            heap.push(Candidate(d2, n.item));
        }

        let diff = target[n.axis] - n.point[n.axis];
        let (near, far) = if diff < 0.0 {
            (n.left, n.right)
        } else {
            (n.right, n.left)
        };
        if let Some(near) = near {
            self.search(near, target, k, heap);
        }
        if let Some(far) = far
            && (heap.len() < k || diff * diff < heap.peek().unwrap().0)
        {
            self.search(far, target, k, heap);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kd_tree_matches_brute_force_k_nearest() {
        // Deterministic pseudo-random points.
        let mut seed = 42u64;
        let mut rand = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as f64 / (1u64 << 31) as f64 * 200.0 - 100.0
        };
        let points: Vec<([f64; 3], usize)> =
            (0..500).map(|i| ([rand(), rand(), rand()], i)).collect();
        let tree = KdTree::build(points.clone());
        for _ in 0..200 {
            let t = [rand(), rand(), rand()];
            let mut got = tree.k_nearest(t, 5);
            got.sort();
            let mut want: Vec<(f64, usize)> = points
                .iter()
                .map(|(p, i)| ((0..3).map(|a| (p[a] - t[a]).powi(2)).sum(), *i))
                .collect();
            want.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut want: Vec<usize> = want[..5].iter().map(|w| w.1).collect();
            want.sort();
            assert_eq!(got, want);
        }
    }

    #[test]
    fn list_colours_name_themselves() {
        let names = Names::bundled();
        let n = names
            .descriptive
            .nearest(Rgb::new(0x87, 0xCE, 0xEB).to_lab());
        assert_eq!(
            (n.name.as_str(), n.hex.as_str(), n.delta_e),
            ("sky blue", "#87CEEB", 0.0)
        );
    }

    #[test]
    fn aliases_are_not_returned_by_lookup() {
        let names = Names::bundled();
        assert_eq!(
            names
                .descriptive
                .nearest(Rgb::new(0, 255, 255).to_lab())
                .name,
            "cyan"
        );
        assert_eq!(
            names
                .descriptive
                .nearest(Rgb::new(128, 128, 128).to_lab())
                .name,
            "grey"
        );
    }

    #[test]
    fn tree_lookup_agrees_with_brute_force() {
        let names = Names::bundled();
        let mut disagreements = 0;
        let mut total = 0;
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    let lab = Rgb::new(r, g, b).to_lab();
                    for list in [&names.descriptive, &names.precise] {
                        total += 1;
                        let fast = list.nearest(lab);
                        let slow = list.nearest_brute_force(lab);
                        if fast.name != slow.name {
                            disagreements += 1;
                            // When they differ, the tree's pick must be about as close.
                            assert!(
                                fast.delta_e - slow.delta_e < 1.5,
                                "{lab:?}: {fast:?} vs {slow:?}"
                            );
                        }
                    }
                }
            }
        }
        // Euclidean pre-selection rarely changes the CIEDE2000 winner.
        assert!(disagreements * 500 < total, "{disagreements}/{total}");
    }

    #[test]
    fn search_ignores_case_spaces_and_punctuation() {
        let names = Names::bundled();
        for q in ["skyblue", "sky blue", "Sky-Blue", "SKY BLUE"] {
            let hits = names.search(q, 5);
            assert_eq!(hits[0].name, "sky blue", "query {q:?}");
            assert_eq!(hits[0].hex, "#87CEEB");
        }
    }

    #[test]
    fn search_treats_gray_and_grey_alike_and_tolerates_typos() {
        let names = Names::bundled();
        assert_eq!(names.search("slate gray", 3)[0].name, "slate grey");
        assert_eq!(names.search("cornflwer blue", 3)[0].name, "cornflower blue");
        assert_eq!(names.search("aqua", 3)[0].name, "aqua");
        assert!(names.search("zzzzzz", 3).is_empty());
        assert!(names.search("  ", 3).is_empty());
    }
}
