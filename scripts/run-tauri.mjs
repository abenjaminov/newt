// Wrapper around `tauri` that scrubs env vars set by other toolchains
// (Android NDK's CC/CXX in particular) so the MSVC build can auto-detect
// the right compiler. Without this, cargo picks up clang++ from the NDK
// and fails to find <windows.h>.

import { spawn } from "node:child_process";

const env = { ...process.env };
for (const k of ["CC", "CXX", "AR", "LD"]) {
  if (env[k]) delete env[k];
}

const isWindows = process.platform === "win32";
const args = process.argv.slice(2);
const cmd = isWindows ? "tauri.cmd" : "tauri";

const child = spawn(cmd, args, {
  env,
  stdio: "inherit",
  shell: isWindows,
});

child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  else process.exit(code ?? 0);
});
