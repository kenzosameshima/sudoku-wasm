/// Backtracking solver: bitmask candidates, most-constrained cell first.
/// MRV makes naked singles (one candidate) fall out for free.
pub struct Board {
    cells: [u8; 81],
    rows: [u16; 9],
    cols: [u16; 9],
    boxes: [u16; 9],
}

fn box_of(i: usize) -> usize {
    i / 27 * 3 + i % 9 / 3
}

impl Board {
    /// None if not 81 chars of 0-9/'.' or if givens conflict.
    pub fn parse(s: &str) -> Option<Board> {
        if s.len() != 81 {
            return None;
        }
        let mut b = Board { cells: [0; 81], rows: [0; 9], cols: [0; 9], boxes: [0; 9] };
        for (i, c) in s.bytes().enumerate() {
            let d = match c {
                b'0' | b'.' => continue,
                b'1'..=b'9' => c - b'0',
                _ => return None,
            };
            let bit = 1u16 << d;
            if (b.rows[i / 9] | b.cols[i % 9] | b.boxes[box_of(i)]) & bit != 0 {
                return None;
            }
            b.place(i, d);
        }
        Some(b)
    }

    fn place(&mut self, i: usize, d: u8) {
        let bit = 1u16 << d;
        self.cells[i] = d;
        self.rows[i / 9] |= bit;
        self.cols[i % 9] |= bit;
        self.boxes[box_of(i)] |= bit;
    }

    fn unplace(&mut self, i: usize) {
        let bit = !(1u16 << self.cells[i]);
        self.cells[i] = 0;
        self.rows[i / 9] &= bit;
        self.cols[i % 9] &= bit;
        self.boxes[box_of(i)] &= bit;
    }

    fn candidates(&self, i: usize) -> u16 {
        !(self.rows[i / 9] | self.cols[i % 9] | self.boxes[box_of(i)]) & 0x3FE
    }

    /// Counts solutions up to `limit`; the first one found is copied to `first`.
    fn search(&mut self, limit: u32, found: &mut u32, first: &mut Option<[u8; 81]>) {
        let mut best = (10, 0, 0u16);
        for i in 0..81 {
            if self.cells[i] == 0 {
                let c = self.candidates(i);
                let n = c.count_ones();
                if n < best.0 {
                    best = (n, i, c);
                    if n <= 1 {
                        break;
                    }
                }
            }
        }
        if best.0 == 10 {
            *found += 1;
            first.get_or_insert(self.cells);
            return;
        }
        let (_, i, mut c) = best;
        while c != 0 && *found < limit {
            let d = c.trailing_zeros() as u8;
            c &= c - 1;
            self.place(i, d);
            self.search(limit, found, first);
            self.unplace(i);
        }
    }
}

fn run(puzzle: &str, limit: u32) -> (u32, Option<[u8; 81]>) {
    let (mut n, mut first) = (0, None);
    if let Some(mut b) = Board::parse(puzzle) {
        b.search(limit, &mut n, &mut first);
    }
    (n, first)
}

pub fn count_solutions(puzzle: &str, limit: u32) -> u32 {
    run(puzzle, limit).0
}

/// Solved grid as 81 digits, or "" if invalid / unsolvable.
pub fn solve(puzzle: &str) -> String {
    run(puzzle, 1).1.map_or(String::new(), |g| g.iter().map(|d| (b'0' + d) as char).collect())
}

/// Well-formed with exactly one solution.
pub fn is_valid(puzzle: &str) -> bool {
    count_solutions(puzzle, 2) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const EASY_SOL: &str = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    // 17-clue, known hard for naive solvers
    const HARD: &str = "000000010400000000020000000000050407008000300001090000300400200050100000000806000";

    #[test]
    fn solves_easy() {
        assert_eq!(solve(EASY), EASY_SOL);
        assert!(is_valid(EASY));
    }

    #[test]
    fn solves_hard_uniquely() {
        let s = solve(HARD);
        assert_eq!(s.len(), 81);
        assert!(is_valid(HARD));
        assert!(!s.contains('0'));
    }

    #[test]
    fn dots_work() {
        assert_eq!(solve(&EASY.replace('0', ".")), EASY_SOL);
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(solve("123"), "");
        assert_eq!(solve(&format!("55{}", "0".repeat(79))), ""); // row conflict
        assert_eq!(count_solutions(&"x".repeat(81), 2), 0);
        // no conflicts among givens but unsolvable: 1-8 in row 0, 9 in col 8 below
        let p = format!("12345678{}{}9{}", "0", "00000000", "0".repeat(63));
        assert_eq!(solve(&p), "");
    }

    #[test]
    fn empty_grid_has_many() {
        assert_eq!(count_solutions(&"0".repeat(81), 2), 2);
        assert!(!is_valid(&"0".repeat(81)));
    }
}
