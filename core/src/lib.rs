mod generator;
mod solver;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn solve(puzzle: &str) -> String {
    solver::solve(puzzle)
}

#[wasm_bindgen]
pub fn is_valid(puzzle: &str) -> bool {
    solver::is_valid(puzzle)
}

#[wasm_bindgen]
pub fn count_solutions(puzzle: &str, limit: u32) -> u32 {
    solver::count_solutions(puzzle, limit)
}

/// difficulty 0..=3 (easy..expert); same seed gives the same puzzle.
#[wasm_bindgen]
pub fn generate(difficulty: u8, seed: u32) -> String {
    generator::generate(difficulty, seed)
}
