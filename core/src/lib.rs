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
