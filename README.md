# Sudoku in Rust + WebAssembly

A sudoku game in the style of sudoku.com. The solver, generator, difficulty rater and hints are written in Rust, compiled to WebAssembly and run entirely in the browser. There is no backend: the build output is plain static files.

Live demo (once the Pages workflow has run): <https://kenzosameshima.github.io/sudoku-wasm/>

## Features

- Generate puzzles by difficulty (easy, medium, hard, expert), rated by the human techniques needed to solve them
- Pencil-mark notes, undo/redo, mistake counter, timer with pause, hints that explain the next logical step
- Progress saved in `localStorage`
- Keyboard: digits, arrows, Backspace/Delete, `N` toggles notes, Ctrl+Z / Ctrl+Y undo and redo
- Responsive layout, dark mode, labelled cells for screen readers

## Architecture

```
core/   Rust crate (cdylib + rlib)
  solver.rs     bitmask backtracking, most-constrained cell first
  generator.rs  seeded RNG, random fill, cell removal while the solution stays unique
  rating.rs     human-style technique engine, used for both rating and hints
  lib.rs        wasm-bindgen exports
web/    Vite + TypeScript UI (grid, input, notes, undo, timer, persistence)
```

The boundary is string-based and coarse (whole puzzle in, whole result out):

| Export | Meaning |
| --- | --- |
| `solve(puzzle)` | solved grid, or `""` if invalid or unsolvable |
| `is_valid(puzzle)` | exactly one solution |
| `count_solutions(puzzle, limit)` | solutions up to `limit` |
| `generate(difficulty, seed)` | unique puzzle, 0 easy to 3 expert, reproducible per seed |
| `rate(puzzle)` | 0 to 3, or 255 if not a unique puzzle |
| `hint(puzzle)` | next logical step as text |

A puzzle is 81 characters, `0` or `.` for empty. The fast solver and the rater are separate on purpose: the solver only needs to be quick, while the rater has to mimic human techniques (naked/hidden singles and pairs, pointing pairs and box/line reduction, X-wing).

## Develop

Requires Rust, `wasm-pack` and Node.

```bash
cargo test --manifest-path core/Cargo.toml   # native tests
cd web
npm install
npm run wasm                                 # build core/pkg
npm run dev                                  # http://localhost:5173
npm run build                                # static site in web/dist
```

## Deploy

`.github/workflows/deploy.yml` runs the tests, builds the WASM and the site, and publishes `web/dist` to GitHub Pages on every push to `main`. In the repository settings, set Pages to deploy from GitHub Actions. The build uses relative asset paths, so it also works on Netlify, Vercel or any static host.

## Notes

- A solver this small would also be instant in plain JavaScript; the point of Rust here is learning, code sharing and a clean core.
- Expert puzzles are those needing X-wing or beyond, or where these techniques get stuck. Techniques such as swordfish are not implemented.
