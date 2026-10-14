import init, { solve, generate, hint } from "../../core/pkg/sudoku_core.js";
import "./style.css";

const $ = (id: string) => document.getElementById(id)!;
const msg = (t: string) => ($("msg").textContent = t);
const SAVE = "sudoku-save";

type Change = { i: number; b: number; n: number }; // cell, value, notes bitmask (bit d)
type Game = {
  puzzle: string; solution: string; level: number;
  board: number[]; notes: number[]; mistakes: number; elapsed: number;
};

let g: Game;
let sel = 0;
let notesMode = false;
let paused = false;
const undoStack: Change[][] = [];
const redoStack: Change[][] = [];

const given = (i: number) => g.puzzle[i] !== "0";
const correct = (i: number) => g.board[i] === +g.solution[i];
const locked = (i: number) => given(i) || correct(i);
const won = () => g.board.every((_, i) => correct(i));
const fmt = (s: number) => `${(s / 60) | 0}:${String(s % 60).padStart(2, "0")}`;
const sees = (a: number, b: number) =>
  (a / 9 | 0) === (b / 9 | 0) || a % 9 === b % 9 ||
  ((a / 27 | 0) === (b / 27 | 0) && ((a % 9) / 3 | 0) === ((b % 9) / 3 | 0));

// localStorage can throw (private mode, blocked data), so never rely on it.
function save() {
  try { localStorage.setItem(SAVE, JSON.stringify(g)); } catch { /* ignore */ }
}
function restore(): Game | null {
  try {
    const s = JSON.parse(localStorage.getItem(SAVE) ?? "null");
    return s && s.puzzle?.length === 81 && s.solution?.length === 81 &&
      s.board?.length === 81 && s.notes?.length === 81 ? s : null;
  } catch { return null; }
}

function start(p: string, level: number) {
  g = {
    puzzle: p, solution: solve(p), level, board: [...p].map(Number),
    notes: Array(81).fill(0), mistakes: 0, elapsed: 0,
  };
  undoStack.length = redoStack.length = 0;
  sel = 0;
  paused = false;
  msg("");
  save();
  render();
}

const newGame = () => {
  const level = +($("level") as HTMLSelectElement).value;
  start(generate(level, (Math.random() * 2 ** 32) >>> 0), level);
};

const cells = Array.from({ length: 81 }, (_, i) => {
  const b = document.createElement("button");
  b.setAttribute("role", "gridcell");
  b.setAttribute("aria-label", `Row ${(i / 9 | 0) + 1}, column ${(i % 9) + 1}`);
  b.onclick = () => ((sel = i), render());
  $("grid").append(b);
  return b;
});

const padButtons = [1, 2, 3, 4, 5, 6, 7, 8, 9].map((d) => {
  const b = document.createElement("button");
  b.textContent = String(d);
  b.setAttribute("aria-label", `Enter ${d}`);
  b.onclick = () => enter(d);
  return b;
});
$("pad").append(...padButtons);

function render() {
  const sd = g.board[sel];
  cells.forEach((c, i) => {
    const b = g.board[i], n = g.notes[i];
    c.className = "cell" +
      (given(i) ? " given" : "") +
      (i === sel ? " sel" : sd && b === sd ? " same" : sees(i, sel) ? " peer" : "") +
      (b && !correct(i) ? " bad" : "") +
      (!b && n ? " hasnotes" : "");
    if (b || !n) c.textContent = b ? String(b) : "";
    else c.innerHTML = [1, 2, 3, 4, 5, 6, 7, 8, 9].map((d) => `<i>${n >> d & 1 ? d : ""}</i>`).join("");
  });
  padButtons.forEach((p, k) => {
    p.disabled = g.board.filter((d, i) => d === k + 1 && correct(i)).length === 9;
  });
  ($("level") as HTMLSelectElement).value = String(g.level);
  $("mistakes").textContent = `Mistakes: ${g.mistakes}`;
  $("time").textContent = fmt(g.elapsed);
  $("grid").classList.toggle("paused", paused);
  $("pause").textContent = paused ? "▶" : "⏸";
  $("pause").setAttribute("aria-label", paused ? "Resume" : "Pause");
  $("notes").textContent = `Notes: ${notesMode ? "on" : "off"}`;
  $("notes").setAttribute("aria-pressed", String(notesMode));
  ($("undo") as HTMLButtonElement).disabled = !undoStack.length;
  ($("redo") as HTMLButtonElement).disabled = !redoStack.length;
}

/** Records the previous state of every touched cell, then applies the new one. */
function commit(next: Change[]) {
  undoStack.push(next.map(({ i }) => ({ i, b: g.board[i], n: g.notes[i] })));
  redoStack.length = 0;
  next.forEach(({ i, b, n }) => ((g.board[i] = b), (g.notes[i] = n)));
}

function step(from: Change[][], to: Change[][]) {
  if (paused || !from.length) return;
  const c = from.pop()!;
  to.push(c.map(({ i }) => ({ i, b: g.board[i], n: g.notes[i] })));
  c.forEach(({ i, b, n }) => ((g.board[i] = b), (g.notes[i] = n)));
  save();
  render();
}

function enter(d: number) {
  if (paused || won() || locked(sel)) return;
  if (d === 0) {
    if (!g.board[sel] && !g.notes[sel]) return;
    commit([{ i: sel, b: 0, n: 0 }]);
  } else if (notesMode) {
    if (g.board[sel]) return;
    commit([{ i: sel, b: 0, n: g.notes[sel] ^ (1 << d) }]);
  } else {
    if (g.board[sel] === d) return;
    // Placing a digit also clears it from the notes of every peer.
    const ch: Change[] = [{ i: sel, b: d, n: 0 }];
    for (let j = 0; j < 81; j++) {
      if (j !== sel && sees(sel, j) && g.notes[j] >> d & 1) ch.push({ i: j, b: g.board[j], n: g.notes[j] & ~(1 << d) });
    }
    commit(ch);
    if (d !== +g.solution[sel]) g.mistakes++;
    msg(won() ? `Solved in ${fmt(g.elapsed)}!` : "");
  }
  save();
  render();
}

document.onkeydown = (e) => {
  if (e.target instanceof HTMLSelectElement) return;
  const mod = e.ctrlKey || e.metaKey;
  if (mod && e.key.toLowerCase() === "z") step(e.shiftKey ? redoStack : undoStack, e.shiftKey ? undoStack : redoStack);
  else if (mod && e.key.toLowerCase() === "y") step(redoStack, undoStack);
  else if (mod || e.altKey) return;
  else if (/^[1-9]$/.test(e.key)) enter(+e.key);
  else if (e.key === "Backspace" || e.key === "Delete" || e.key === "0") enter(0);
  else if (e.key.toLowerCase() === "n") { notesMode = !notesMode; render(); }
  else {
    const d = ({ ArrowLeft: -1, ArrowRight: 1, ArrowUp: -9, ArrowDown: 9 } as Record<string, number>)[e.key];
    if (!d) return;
    const n = sel + d;
    if (n >= 0 && n < 81 && !(Math.abs(d) === 1 && (n / 9 | 0) !== (sel / 9 | 0))) sel = n;
    cells[sel].focus();
    render();
  }
  e.preventDefault();
};

const setPaused = (p: boolean) => { paused = p; render(); };
$("pause").onclick = () => setPaused(!paused);
document.onvisibilitychange = () => document.hidden && setPaused(true);
setInterval(() => {
  if (paused || won()) return;
  $("time").textContent = fmt(++g.elapsed);
  if (g.elapsed % 5 === 0) save();
}, 1000);

$("undo").onclick = () => step(undoStack, redoStack);
$("redo").onclick = () => step(redoStack, undoStack);
$("erase").onclick = () => enter(0);
$("notes").onclick = () => ((notesMode = !notesMode), render());
$("new").onclick = newGame;
$("hint").onclick = () => {
  // Hint from the correct entries only, so a wrong digit cannot confuse the engine.
  const p = g.board.map((d, i) => (correct(i) ? d : 0)).join("");
  msg(hint(p) || "No logical step found.");
};
$("solve").onclick = () => {
  commit(g.board.map((_, i) => ({ i, b: +g.solution[i], n: 0 })));
  msg("Solved.");
  save();
  render();
};

await init();
const saved = restore();
if (saved) { g = saved; render(); } else newGame();
