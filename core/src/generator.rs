use crate::solver::{count_solutions, Board};

/// xorshift64*, seeded through splitmix64 so small seeds still diverge.
struct Rng(u64);

impl Rng {
    fn new(seed: u32) -> Rng {
        let mut z = (seed as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Rng((z ^ (z >> 31)) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            v.swap(i, (self.next() % (i as u64 + 1)) as usize);
        }
    }
}

/// Randomized backtracking fill, cell by cell in order.
fn fill(b: &mut Board, i: usize, rng: &mut Rng) -> bool {
    if i == 81 {
        return true;
    }
    let mut ds: Vec<u8> = (1..=9).filter(|d| b.candidates(i) >> d & 1 == 1).collect();
    rng.shuffle(&mut ds);
    for d in ds {
        b.place(i, d);
        if fill(b, i + 1, rng) {
            return true;
        }
        b.unplace(i);
    }
    false
}

/// Clues to aim for: 0 easy, 1 medium, 2 hard, 3 expert (anything higher = expert).
/// Targets are a floor on clues, not a rating; real difficulty comes with milestone 5.
fn target_clues(difficulty: u8) -> usize {
    [40, 34, 28, 24][difficulty.min(3) as usize]
}

/// Puzzle with a unique solution; same (difficulty, seed) gives the same puzzle.
/// May keep more clues than the target if no further cell can be removed uniquely.
pub fn generate(difficulty: u8, seed: u32) -> String {
    let mut rng = Rng::new(seed);
    let mut b = Board::parse(&"0".repeat(81)).unwrap();
    assert!(fill(&mut b, 0, &mut rng));
    let mut cells = b.cells().to_vec();
    let mut order: Vec<usize> = (0..81).collect();
    rng.shuffle(&mut order);
    let mut clues = 81;
    for i in order {
        if clues <= target_clues(difficulty) {
            break;
        }
        let d = cells[i];
        cells[i] = 0;
        let s: String = cells.iter().map(|d| (b'0' + d) as char).collect();
        if count_solutions(&s, 2) == 1 {
            clues -= 1;
        } else {
            cells[i] = d;
        }
    }
    cells.iter().map(|d| (b'0' + d) as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::is_valid;

    #[test]
    fn unique_and_hits_target() {
        for (diff, seed) in [(0, 1), (1, 2), (2, 3), (3, 4)] {
            let p = generate(diff, seed);
            assert!(is_valid(&p), "diff {diff} seed {seed}");
            let clues = p.bytes().filter(|&c| c != b'0').count();
            assert!(clues >= target_clues(diff) && clues <= target_clues(diff) + 3, "{clues} clues");
        }
    }

    #[test]
    fn seed_is_reproducible() {
        assert_eq!(generate(1, 42), generate(1, 42));
        assert_ne!(generate(1, 42), generate(1, 43));
    }
}
