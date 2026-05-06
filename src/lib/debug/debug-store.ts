// Tracks the local view of all debug sessions: stopped state, current
// thread, current stack/scopes/variables. Phase 1 keeps this small — UI
// and MCP server are wired against the same store.

import { writable, derived } from "svelte/store";
import * as dbg from "./debug-client";

export type StoppedReason =
  | "step"
  | "breakpoint"
  | "exception"
  | "pause"
  | "entry"
  | "goto"
  | "function-breakpoint"
  | "data-breakpoint"
  | "instruction-breakpoint"
  | string;

export type SessionState = {
  id: number;
  /** Adapter id (e.g. "debugpy"). */
  adapterId: string;
  /** Most recent stopped event, if any. */
  stopped:
    | {
        reason: StoppedReason;
        threadId?: number;
        description?: string;
        text?: string;
      }
    | null;
  terminated: boolean;
};

const sessionsRw = writable<Record<number, SessionState>>({});

export const sessions = {
  subscribe: sessionsRw.subscribe,
};

/**
 * Launch a debug session and start mirroring its events into the store.
 */
export async function launch(args: dbg.DebugLaunchArgs): Promise<number> {
  const id = await dbg.launch(args);
  sessionsRw.update((s) => ({
    ...s,
    [id]: { id, adapterId: args.adapterId, stopped: null, terminated: false },
  }));
  void dbg.subscribe(id, (ev) => onEvent(id, ev));
  return id;
}

function onEvent(id: number, ev: dbg.DapEventEnvelope) {
  switch (ev.event) {
    case "stopped": {
      const body = (ev.body ?? {}) as {
        reason?: string;
        threadId?: number;
        description?: string;
        text?: string;
      };
      sessionsRw.update((s) => {
        const cur = s[id];
        if (!cur) return s;
        return {
          ...s,
          [id]: {
            ...cur,
            stopped: {
              reason: body.reason ?? "unknown",
              threadId: body.threadId,
              description: body.description,
              text: body.text,
            },
          },
        };
      });
      break;
    }
    case "continued": {
      sessionsRw.update((s) => {
        const cur = s[id];
        if (!cur) return s;
        return { ...s, [id]: { ...cur, stopped: null } };
      });
      break;
    }
    case "terminated":
    case "exited": {
      sessionsRw.update((s) => {
        const cur = s[id];
        if (!cur) return s;
        return { ...s, [id]: { ...cur, terminated: true } };
      });
      break;
    }
    // output / thread / breakpoint / module / loadedSource etc. — add as
    // the UI / MCP server needs them.
    default:
      break;
  }
}

export const activeSessions = derived(sessionsRw, ($s) =>
  Object.values($s).filter((x) => !x.terminated),
);

// Re-export the raw client for callers that just need to make a request.
export { dbg as client };
