// Lightweight fuzzy scorer: rewards substring matches, sequential characters,
// matches at word boundaries, and (lightly) shorter strings. Returns -1 for
// no-match so callers can filter cleanly.

export type Match = { score: number; ranges: Array<[number, number]> };

const WORD_BOUNDARY = /[\s\-_/.\\]/;

export function fuzzyScore(query: string, text: string): Match | null {
  if (!query) return { score: 0, ranges: [] };
  const q = query.toLowerCase();
  const t = text.toLowerCase();

  // Exact substring — strongest signal.
  const idx = t.indexOf(q);
  if (idx >= 0) {
    const lengthPenalty = (text.length - query.length) * 0.05;
    let bonus = 0;
    if (idx === 0) bonus += 30;
    else if (WORD_BOUNDARY.test(t[idx - 1] ?? "")) bonus += 20;
    return {
      score: 200 + bonus - idx - lengthPenalty,
      ranges: [[idx, idx + query.length]],
    };
  }

  // Subsequence: every query char must appear in order in text.
  let qi = 0;
  let prev = -2;
  let score = 0;
  const ranges: Array<[number, number]> = [];
  let runStart = -1;
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] !== q[qi]) {
      if (runStart >= 0) {
        ranges.push([runStart, ti]);
        runStart = -1;
      }
      continue;
    }
    if (ti === prev + 1) {
      score += 6; // consecutive bonus
    } else {
      score += 1;
    }
    if (ti === 0 || WORD_BOUNDARY.test(t[ti - 1] ?? "")) {
      score += 4;
    }
    if (runStart < 0) runStart = ti;
    prev = ti;
    qi++;
  }
  if (runStart >= 0) ranges.push([runStart, prev + 1]);
  if (qi < q.length) return null;
  return { score: score - text.length * 0.05, ranges };
}
