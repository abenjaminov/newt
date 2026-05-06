import { Channel, invoke } from "@tauri-apps/api/core";

export type SpawnOpts = {
  program: string;
  args: string[];
  cwd: string | null;
  cols: number;
  rows: number;
  label: string;
  onData: (data: string) => void;
  onExit: (code: number) => void;
};

export type PtyHandle = {
  id: number;
  write: (data: string) => Promise<void>;
  resize: (cols: number, rows: number) => Promise<void>;
  kill: () => Promise<void>;
};

export async function spawnPty(opts: SpawnOpts): Promise<PtyHandle> {
  const onData = new Channel<string>();
  onData.onmessage = (msg) => opts.onData(msg);

  const onExit = new Channel<number>();
  onExit.onmessage = (code) => opts.onExit(code);

  const id = await invoke<number>("spawn_pty", {
    program: opts.program,
    args: opts.args,
    cwd: opts.cwd,
    cols: opts.cols,
    rows: opts.rows,
    label: opts.label,
    onData,
    onExit,
  });

  return {
    id,
    write: (data) => invoke("write_pty", { id, data }),
    resize: (cols, rows) => invoke("resize_pty", { id, cols, rows }),
    kill: () => invoke("kill_pty", { id }),
  };
}

export type ShellInfo = {
  id: string;
  label: string;
  program: string;
  args: string[];
  kind: "cmd" | "powershell" | "pwsh" | "wsl" | "posix";
  default: boolean;
};

export async function listShells(): Promise<ShellInfo[]> {
  return invoke<ShellInfo[]>("list_shells");
}

export type PtyEntry = {
  uid: number;
  os_pid: number | null;
  label: string;
};

export async function listPtys(): Promise<PtyEntry[]> {
  return invoke<PtyEntry[]>("list_ptys");
}
