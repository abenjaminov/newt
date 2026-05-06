import { writable } from "svelte/store";
import type { ShellInfo } from "./pty-client";

export type TermSession = {
  id: number;             // pty id from Rust
  uid: number;            // ui id (incremented locally)
  label: string;
  shell: ShellInfo;
  exited: boolean;
  exitCode: number | null;
};

type TermState = {
  sessions: TermSession[];
  activeUid: number | null;
};

let nextUid = 1;

function create() {
  const { subscribe, update } = writable<TermState>({
    sessions: [],
    activeUid: null,
  });

  return {
    subscribe,
    add(session: Omit<TermSession, "uid" | "exited" | "exitCode">) {
      const uid = nextUid++;
      update((s) => ({
        sessions: [
          ...s.sessions,
          { ...session, uid, exited: false, exitCode: null },
        ],
        activeUid: uid,
      }));
      return uid;
    },
    activate(uid: number) {
      update((s) => ({ ...s, activeUid: uid }));
    },
    markExit(uid: number, code: number) {
      update((s) => ({
        ...s,
        sessions: s.sessions.map((t) =>
          t.uid === uid ? { ...t, exited: true, exitCode: code } : t,
        ),
      }));
    },
    remove(uid: number) {
      update((s) => {
        const sessions = s.sessions.filter((t) => t.uid !== uid);
        let active = s.activeUid;
        if (active === uid) {
          active = sessions.length ? sessions[sessions.length - 1].uid : null;
        }
        return { sessions, activeUid: active };
      });
    },
  };
}

export const terminals = create();
