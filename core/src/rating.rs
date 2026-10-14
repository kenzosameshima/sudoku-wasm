//! Human-style technique engine. Separate from the fast solver on purpose:
//! it applies one logical step at a time so it can both rate and explain.
use crate::solver::is_valid;

const ALL: u16 = 0x3FE; // bits 1..=9

struct State {
    cells: [u8; 81],
    cand: [u16; 81],
    units: [[usize; 9]; 27], // 0-8 rows, 9-17 columns, 18-26 boxes
}

fn rc(i: usize) -> String {
    format!("r{}c{}", i / 9 + 1, i % 9 + 1)
}

fn unit_name(u: usize) -> String {
    format!("{} {}", ["row", "column", "box"][u / 9], u % 9 + 1)
}

fn digits(mask: u16) -> String {
    (1..=9).filter(|d| mask >> d & 1 == 1).map(|d| d.to_string()).collect::<Vec<_>>().join("/")
}

impl State {
    fn new(p: &str) -> State {
        let mut units = [[0; 9]; 27];
        for i in 0..81 {
            for (u, k) in [(i / 9, i % 9), (9 + i % 9, i / 9), (18 + i / 27 * 3 + i % 9 / 3, i / 9 % 3 * 3 + i % 3)] {
                units[u][k] = i;
            }
        }
        let mut s = State { cells: [0; 81], cand: [ALL; 81], units };
        for (i, c) in p.bytes().enumerate() {
            if (b'1'..=b'9').contains(&c) {
                s.set(i, c - b'0');
            }
        }
        s
    }

    fn units_of(i: usize) -> [usize; 3] {
        [i / 9, 9 + i % 9, 18 + i / 27 * 3 + i % 9 / 3]
    }

    fn set(&mut self, i: usize, d: u8) {
        self.cells[i] = d;
        self.cand[i] = 0;
        for u in State::units_of(i) {
            for k in 0..9 {
                self.cand[self.units[u][k]] &= !(1 << d);
            }
        }
    }

    /// Removes `mask` from cell i's candidates; true if anything changed.
    fn elim(&mut self, i: usize, mask: u16) -> bool {
        let hit = self.cand[i] & mask != 0;
        self.cand[i] &= !mask;
        hit
    }

    fn solved(&self) -> bool {
        self.cells.iter().all(|&d| d != 0)
    }

    /// Cells of unit u that still have digit d as a candidate.
    fn spots(&self, u: usize, d: usize) -> Vec<usize> {
        self.units[u].iter().copied().filter(|&i| self.cand[i] >> d & 1 == 1).collect()
    }
}

type Technique = fn(&mut State) -> Option<String>;

/// In rating order; index + 1 is the technique's level.
const TECHNIQUES: [Technique; 6] = [naked_single, hidden_single, naked_pair, hidden_pair, pointing, x_wing];

fn naked_single(s: &mut State) -> Option<String> {
    let i = (0..81).find(|&i| s.cand[i].count_ones() == 1)?;
    let d = s.cand[i].trailing_zeros() as u8;
    s.set(i, d);
    Some(format!("Naked single: {} can only be {}.", rc(i), d))
}

fn hidden_single(s: &mut State) -> Option<String> {
    for u in 0..27 {
        for d in 1..=9 {
            if let [i] = s.spots(u, d)[..] {
                s.set(i, d as u8);
                return Some(format!("Hidden single: {} only fits at {} in {}.", d, rc(i), unit_name(u)));
            }
        }
    }
    None
}

fn naked_pair(s: &mut State) -> Option<String> {
    for u in 0..27 {
        for a in 0..9 {
            let (ia, c) = (s.units[u][a], s.cand[s.units[u][a]]);
            if c.count_ones() != 2 {
                continue;
            }
            for b in a + 1..9 {
                if s.cand[s.units[u][b]] != c {
                    continue;
                }
                let mut changed = false;
                for k in (0..9).filter(|&k| k != a && k != b) {
                    changed |= s.elim(s.units[u][k], c);
                }
                if changed {
                    return Some(format!(
                        "Naked pair {} at {} and {} in {}: remove them from the other cells.",
                        digits(c), rc(ia), rc(s.units[u][b]), unit_name(u)
                    ));
                }
            }
        }
    }
    None
}

fn hidden_pair(s: &mut State) -> Option<String> {
    for u in 0..27 {
        for d1 in 1..=9 {
            let p = s.spots(u, d1);
            if p.len() != 2 {
                continue;
            }
            for d2 in d1 + 1..=9 {
                if s.spots(u, d2) != p {
                    continue;
                }
                let keep = (1 << d1) | (1 << d2);
                let mut changed = false;
                for &i in &p {
                    changed |= s.elim(i, ALL & !keep);
                }
                if changed {
                    return Some(format!(
                        "Hidden pair {}/{} at {} and {} in {}: remove other candidates there.",
                        d1, d2, rc(p[0]), rc(p[1]), unit_name(u)
                    ));
                }
            }
        }
    }
    None
}

/// Pointing pairs/triples and box/line reduction: a digit confined to the
/// overlap of two units of different type is cleared from the rest of the other one.
fn pointing(s: &mut State) -> Option<String> {
    for a in 0..27 {
        for d in 1..=9 {
            let p = s.spots(a, d);
            if p.len() < 2 {
                continue;
            }
            for b in (0..27).filter(|b| b / 9 != a / 9) {
                if !p.iter().all(|i| s.units[b].contains(i)) {
                    continue;
                }
                let mut changed = false;
                for k in 0..9 {
                    let j = s.units[b][k];
                    if !p.contains(&j) {
                        changed |= s.elim(j, 1 << d);
                    }
                }
                if changed {
                    return Some(format!(
                        "In {}, {} must be in {}, so remove it from the rest of {}.",
                        unit_name(a), d, unit_name(b), unit_name(b)
                    ));
                }
            }
        }
    }
    None
}

fn x_wing(s: &mut State) -> Option<String> {
    for base in [0, 9] {
        let cross = 9 - base;
        for d in 1..=9 {
            let slots = |s: &State, l: usize| -> Vec<usize> {
                (0..9).filter(|&k| s.cand[s.units[base + l][k]] >> d & 1 == 1).collect()
            };
            for l1 in 0..9 {
                let p = slots(s, l1);
                if p.len() != 2 {
                    continue;
                }
                for l2 in l1 + 1..9 {
                    if slots(s, l2) != p {
                        continue;
                    }
                    let mut changed = false;
                    for &c in &p {
                        for k in (0..9).filter(|&k| k != l1 && k != l2) {
                            changed |= s.elim(s.units[cross + c][k], 1 << d);
                        }
                    }
                    if changed {
                        return Some(format!(
                            "X-wing on {} in {} and {}: remove it from the other cells of those {}s.",
                            d, unit_name(base + l1), unit_name(base + l2), ["column", "row"][base / 9]
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Hardest technique level needed (1-6), or 7 if logic alone gets stuck.
fn level(puzzle: &str) -> u8 {
    let mut s = State::new(puzzle);
    let mut hardest = 1;
    while !s.solved() {
        match TECHNIQUES.iter().enumerate().find_map(|(t, f)| f(&mut s).map(|_| t as u8 + 1)) {
            Some(t) => hardest = hardest.max(t),
            None => return 7,
        }
    }
    hardest
}

/// 0 easy (singles), 1 medium (pairs), 2 hard (pointing/box-line), 3 expert
/// (X-wing or beyond). Caller guarantees `puzzle` is a valid unique puzzle.
pub fn difficulty(puzzle: &str) -> u8 {
    match level(puzzle) {
        1..=2 => 0,
        3..=4 => 1,
        5 => 2,
        _ => 3,
    }
}

/// `difficulty`, or 255 if the puzzle is malformed or lacks a unique solution.
pub fn rate(puzzle: &str) -> u8 {
    if is_valid(puzzle) { difficulty(puzzle) } else { 255 }
}

/// Describes the next logical step, or "" if none applies.
pub fn hint(puzzle: &str) -> String {
    let mut s = State::new(puzzle);
    TECHNIQUES.iter().find_map(|f| f(&mut s)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const HARD: &str = "800000000003600000070090200050007000000045700000100030001000068008500010090000400";

    #[test]
    fn easy_is_singles_only() {
        assert_eq!(rate(EASY), 0);
        assert!(hint(EASY).contains("single"));
    }

    #[test]
    fn inkala_puzzle_is_expert() {
        assert_eq!(rate(HARD), 3);
    }

    #[test]
    fn invalid_is_255() {
        assert_eq!(rate("123"), 255);
        assert_eq!(rate(&"0".repeat(81)), 255);
    }

    #[test]
    fn x_wing_fires() {
        // Standard X-wing teaching puzzle; the engine must need level 6 for it.
        let p = "100000569492056108056109240009640801064010000218035604040500016905061402621000005";
        assert!(is_valid(p));
        assert_eq!(level(p), 6);
    }
}

