use crate::rating::difficulty as rate;
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

/// Clue count to reach before the rating is even considered.
fn clue_floor(difficulty: u8) -> usize {
    [40, 34, 28, 24][difficulty as usize]
}

fn text(cells: &[u8]) -> String {
    cells.iter().map(|d| (b'0' + d) as char).collect()
}

/// One fill + removal pass: stop at the first unique puzzle that is at or
/// below the clue floor and at least as hard as asked.
fn attempt(difficulty: u8, rng: &mut Rng) -> String {
    let mut b = Board::parse(&"0".repeat(81)).unwrap();
    assert!(fill(&mut b, 0, rng));
    let mut cells = *b.cells();
    let mut order: Vec<usize> = (0..81).collect();
    rng.shuffle(&mut order);
    let mut clues = 81;
    for i in order {
        let d = cells[i];
        cells[i] = 0;
        if count_solutions(&text(&cells), 2) == 1 {
            clues -= 1;
            if clues <= clue_floor(difficulty) && rate(&text(&cells)) >= difficulty {
                break;
            }
        } else {
            cells[i] = d;
        }
    }
    text(&cells)
}

/// Puzzle with a unique solution rated `difficulty` (0 easy .. 3 expert, higher = expert).
/// Same (difficulty, seed) gives the same puzzle. Retries fresh grids until the rating
/// matches; after 200 misses returns the last attempt (rating may differ).
pub fn generate(difficulty: u8, seed: u32) -> String {
    let difficulty = difficulty.min(3);
    let mut rng = Rng::new(seed);
    let mut p = attempt(difficulty, &mut rng);
    for _ in 1..200 {
        if rate(&p) == difficulty {
            break;
        }
        p = attempt(difficulty, &mut rng);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::is_valid;

    #[test]
    fn unique_and_rated_as_asked() {
        for (diff, seed) in [(0, 1), (1, 2), (2, 3), (3, 4), (1, 5), (2, 6), (2, 4)] {
            let p = generate(diff, seed);
            assert!(is_valid(&p), "diff {diff} seed {seed}");
            assert_eq!(rate(&p), diff, "diff {diff} seed {seed}");
        }
    }

    #[test]
    fn seed_is_reproducible() {
        assert_eq!(generate(1, 42), generate(1, 42));
        assert_ne!(generate(1, 42), generate(1, 43));
    }
}
