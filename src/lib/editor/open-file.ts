import { invoke } from "@tauri-apps/api/core";
import { fileKind } from "./languages";
import { tabs } from "./tabs-store";

type ImageInfo = { data_url: string; size: number };

function basename(p: string): string {
  return p.split(/[\\/]/).filter(Boolean).pop() ?? p;
}

export async function openFileAtPath(absPath: string): Promise<{ ok: true } | { ok: false; error: string }> {
  const name = basename(absPath);
  const kind = fileKind(name);
  try {
    if (kind === "image") {
      const info = await invoke<ImageInfo>("read_image", { path: absPath });
      tabs.open({
        path: absPath,
        kind,
        content: "",
        imageDataUrl: info.data_url,
        imageSize: info.size,
      });
    } else if (kind === "svg") {
      const [content, info] = await Promise.all([
        invoke<string>("read_file", { path: absPath }),
        invoke<ImageInfo>("read_image", { path: absPath }),
      ]);
      tabs.open({
        path: absPath,
        kind,
        content,
        imageDataUrl: info.data_url,
        imageSize: info.size,
      });
    } else {
      const content = await invoke<string>("read_file", { path: absPath });
      tabs.open({ path: absPath, kind: "text", content });
    }
    return { ok: true };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}
