// Phase 1 frontend client for the Rust DAP engine. Thin wrapper around
// `invoke` and the per-session event channel emitted by the backend.
//
// The MCP server (phase 2) will expose the same operations via tools, so
// any new method added here should also be added there.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type DebugLaunchArgs = {
  /** Adapter command (e.g. "python", "node", "lldb-dap"). */
  adapterProgram: string;
  /** Adapter arguments (e.g. ["-m", "debugpy.adapter"]). */
  adapterArgs?: string[];
  /** `adapterID` passed in the `initialize` request. */
  adapterId: string;
  /** Adapter-specific `launch` (or `attach`) arguments — passed through. */
  launchArgs: Record<string, unknown>;
  /** Use `attach` instead of `launch`. */
  attach?: boolean;
};

export type DapEventEnvelope = {
  event: string;
  body?: unknown;
};

export async function launch(args: DebugLaunchArgs): Promise<number> {
  // Cargo names are snake_case; we convert at the boundary.
  return invoke<number>("debug_launch", {
    args: {
      adapter_program: args.adapterProgram,
      adapter_args: args.adapterArgs ?? [],
      adapter_id: args.adapterId,
      launch_args: args.launchArgs,
      attach: args.attach ?? false,
    },
  });
}

export function configurationDone(id: number): Promise<void> {
  return invoke("debug_configuration_done", { id });
}

export function setBreakpoints(
  id: number,
  sourcePath: string,
  lines: number[],
): Promise<unknown> {
  return invoke("debug_set_breakpoints", {
    id,
    args: { source_path: sourcePath, lines },
  });
}

export const cont = (id: number, threadId: number) =>
  invoke<void>("debug_continue", { id, threadId });

export const stepOver = (id: number, threadId: number) =>
  invoke<void>("debug_step_over", { id, threadId });

export const stepIn = (id: number, threadId: number) =>
  invoke<void>("debug_step_in", { id, threadId });

export const stepOut = (id: number, threadId: number) =>
  invoke<void>("debug_step_out", { id, threadId });

export const pause = (id: number, threadId: number) =>
  invoke<void>("debug_pause", { id, threadId });

export const threads = (id: number) =>
  invoke<unknown>("debug_threads", { id });

export const stackTrace = (id: number, threadId: number) =>
  invoke<unknown>("debug_stack_trace", { id, threadId });

export const scopes = (id: number, frameId: number) =>
  invoke<unknown>("debug_scopes", { id, frameId });

export const variables = (id: number, variablesReference: number) =>
  invoke<unknown>("debug_variables", { id, variablesReference });

export const evaluate = (
  id: number,
  expression: string,
  frameId?: number,
  context?: "watch" | "repl" | "hover" | "clipboard" | "variables",
) => invoke<unknown>("debug_evaluate", { id, expression, frameId, context });

export const terminate = (id: number) =>
  invoke<void>("debug_terminate", { id });

export const listSessions = () =>
  invoke<number[]>("debug_list_sessions");

export type InspectArgs = {
  threadId: number;
  /** Number of frames to include (default 5). */
  frameCount?: number;
  /** Max depth for variable trees (default 2). */
  maxVarDepth?: number;
  /** Max children per container (default 50). */
  maxChildrenPerScope?: number;
  /** Watch expressions evaluated against the top frame. */
  watches?: string[];
};

export type InspectVariable = {
  name: string;
  value: string;
  type: string | null;
  variables_reference: number;
  children: InspectVariable[];
  truncated_descendants?: boolean;
  truncated_siblings?: boolean;
};

export type InspectScope = {
  name: string;
  expensive: boolean;
  variables_reference: number;
  variables: InspectVariable[];
};

export type InspectFrame = {
  id: number;
  name: string;
  source: { name?: string; path?: string } | null;
  line: number;
  column: number;
  scopes: InspectScope[];
};

export type InspectSnapshot = {
  thread_id: number;
  frames: InspectFrame[];
  watches: Array<
    | { expression: string; ok: true; result: unknown }
    | { expression: string; ok: false; error: string }
  >;
};

/**
 * One-shot snapshot of the stopped session — frames + scopes + variables +
 * watches in a single round trip. Designed for LLM consumption.
 */
export function inspect(id: number, args: InspectArgs): Promise<InspectSnapshot> {
  return invoke<InspectSnapshot>("debug_inspect", {
    id,
    args: {
      thread_id: args.threadId,
      frame_count: args.frameCount ?? 5,
      max_var_depth: args.maxVarDepth ?? 2,
      max_children_per_scope: args.maxChildrenPerScope ?? 50,
      watches: args.watches ?? [],
    },
  });
}

/**
 * Subscribe to events emitted by a single debug session. Each session emits
 * on the channel `dap:event:<id>`. Returns an unlisten function.
 */
export async function subscribe(
  id: number,
  handler: (ev: DapEventEnvelope) => void,
): Promise<UnlistenFn> {
  return listen<DapEventEnvelope>(`dap:event:${id}`, (e) => handler(e.payload));
}
