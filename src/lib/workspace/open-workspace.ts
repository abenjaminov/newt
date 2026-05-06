import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { get } from "svelte/store";
import { addRecent } from "../recent/recent";
import { tabs } from "../editor/tabs-store";
import { workspace } from "./workspace-store";

/** Show a folder picker; if the user picks one, switch to it. */
export async function pickAndOpenFolder(): Promise<boolean> {
  const result = await openDialog({ directory: true, multiple: false });
  if (typeof result !== "string") return false;
  await openFolder(result);
  return true;
}

/** Switch the workspace to `path`, persist a recent entry, and close any tabs
 * whose file lives outside the new folder. */
export async function openFolder(path: string): Promise<void> {
  await addRecent(path);
  const name = path.split(/[\\/]/).filter(Boolean).pop() ?? path;
  pruneForeignTabs(path);
  workspace.set({ rootPath: path, rootName: name });
}

function pruneForeignTabs(rootPath: string) {
  const norm = (s: string) =>
    s.replace(/\\/g, "/").replace(/\/+$/, "");
  const root = norm(rootPath);
  const isInsideRoot = (p: string) => {
    const np = norm(p);
    return np === root || np.startsWith(root + "/");
  };

  const snapshot = get(tabs).tabs;
  for (const t of snapshot) {
    let belongs = false;
    if (t.kind === "diff" && t.diffMeta) {
      belongs = isInsideRoot(t.diffMeta.repo);
    } else {
      belongs = isInsideRoot(t.path);
    }
    if (!belongs) tabs.close(t.path);
  }
}
