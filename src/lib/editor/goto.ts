import { writable } from "svelte/store";
import { openFileAtPath } from "./open-file";

export type GotoTarget = {
  path: string;
  line: number; // 1-based
  colStart?: number; // 0-based, optional column to select from
  colEnd?: number; // 0-based, optional column to select to
  // Bumped on every request so subscribers re-fire even when target equals last.
  nonce: number;
};

export const pendingGoto = writable<GotoTarget | null>(null);

let nonce = 0;

export async function openFileAt(
  path: string,
  line: number,
  colStart?: number,
  colEnd?: number,
): Promise<{ ok: true } | { ok: false; error: string }> {
  const r = await openFileAtPath(path);
  if (!r.ok) return r;
  pendingGoto.set({ path, line, colStart, colEnd, nonce: ++nonce });
  return { ok: true };
}
