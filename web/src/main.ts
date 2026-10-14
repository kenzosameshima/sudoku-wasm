import init, { solve, generate } from "../../core/pkg/sudoku_core.js";
import "./style.css";

let puzzle = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

const $ = (id: string) => document.getElementById(id)!;
const msg = (t: string) => ($("msg").textContent = t);

const board = [...puzzle].map(Number); // 0 = empty
const given = board.map((d) => d !== 0);

function load(p: string) {
  puzzle = p;
  [...p].forEach((d, i) => ((board[i] = +d), (given[i] = d !== "0")));
  sel = 0;
  msg("");
  render();
}
let sel = 0;

const cells = Array.from({ length: 81 }, (_, i) => {
  const b = document.createElement("button");
  b.className = "cell";
  b.setAttribute("role", "gridcell");
  b.setAttribute("aria-label", `Row ${(i / 9 | 0) + 1}, column ${(i % 9) + 1}`);
  b.onclick = () => ((sel = i), render());
  $("grid").append(b);
  return b;
});

const sees = (a: number, b: number) =>
  (a / 9 | 0) === (b / 9 | 0) || a % 9 === b % 9 ||
  ((a / 27 | 0) === (b / 27 | 0) && ((a % 9) / 3 | 0) === ((b % 9) / 3 | 0));

const conflict = (i: number) =>
  board[i] !== 0 && board.some((d, j) => j !== i && d === board[i] && sees(i, j));

function render() {
  cells.forEach((c, i) => {
    c.textContent = board[i] ? String(board[i]) : "";
    c.className = "cell" +
      (given[i] ? " given" : "") +
      (i === sel ? " sel" : board[sel] && board[i] === board[sel] ? " same" : sees(i, sel) ? " peer" : "") +
      (conflict(i) ? " bad" : "");
  });
}

function set(d: number) {
  if (given[sel]) return;
  board[sel] = d;
  msg("");
  render();
}

const str = () => board.join("");

$("pad").append(...[1, 2, 3, 4, 5, 6, 7, 8, 9, 0].map((d) => {
  const b = document.createElement("button");
  b.textContent = d ? String(d) : "⌫";
  b.setAttribute("aria-label", d ? `Enter ${d}` : "Clear cell");
  b.onclick = () => set(d);
  return b;
}));

document.onkeydown = (e) => {
  if (/^[1-9]$/.test(e.key)) set(+e.key);
  else if (e.key === "Backspace" || e.key === "Delete" || e.key === "0") set(0);
  else {
    const d = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -9, ArrowDown: 9 }[e.key];
    if (!d) return;
    const n = sel + d;
    if (n >= 0 && n < 81 && !(Math.abs(d) === 1 && (n / 9 | 0) !== (sel / 9 | 0))) sel = n;
    cells[sel].focus();
    e.preventDefault();
  }
  render();
};

await init();

$("solve").onclick = () => {
  const s = solve(str());
  if (!s) return msg("No solution.");
  s.split("").forEach((d, i) => (board[i] = +d));
  render();
  msg("Solved.");
};

$("check").onclick = () => {
  const s = solve(str());
  if (!s) return msg("Not solvable — there's a mistake.");
  const wrong = board.some((d, i) => d && d !== +s[i]);
  msg(wrong ? "Some entries differ from the solution." : board.every(Boolean) ? "Correct!" : "So far so good.");
};

$("reset").onclick = () => load(puzzle);

$("new").onclick = () =>
  load(generate(+($("level") as HTMLSelectElement).value, (Math.random() * 2 ** 32) >>> 0));

render();
