import { LazyStore } from "@tauri-apps/plugin-store";

const store = new LazyStore("recents.json");
const KEY = "recentFolders";
const MAX_RECENTS = 10;

export type RecentFolder = {
  path: string;
  name: string;
  lastOpened: number;
};

export async function getRecents(): Promise<RecentFolder[]> {
  const list = (await store.get<RecentFolder[]>(KEY)) ?? [];
  return list.sort((a, b) => b.lastOpened - a.lastOpened);
}

export async function addRecent(path: string): Promise<void> {
  const name = path.split(/[\\/]/).filter(Boolean).pop() ?? path;
  const existing = (await store.get<RecentFolder[]>(KEY)) ?? [];
  const filtered = existing.filter((r) => r.path !== path);
  const next: RecentFolder[] = [
    { path, name, lastOpened: Date.now() },
    ...filtered,
  ].slice(0, MAX_RECENTS);
  await store.set(KEY, next);
  await store.save();
}

export async function removeRecent(path: string): Promise<void> {
  const existing = (await store.get<RecentFolder[]>(KEY)) ?? [];
  await store.set(
    KEY,
    existing.filter((r) => r.path !== path),
  );
  await store.save();
}
