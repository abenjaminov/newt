export type DirEntry = {
  name: string;
  path: string;
  is_dir: boolean;
};

export type TreeNode = DirEntry & {
  children?: TreeNode[];
  expanded?: boolean;
  loaded?: boolean;
};

let _root = $state<TreeNode | null>(null);
let _selected = $state<string | null>(null);
let _renaming = $state<string | null>(null); // path of node being renamed inline

function basename(p: string): string {
  return p.split(/[\\/]/).filter(Boolean).pop() ?? p;
}

function joinPath(parent: string, name: string): string {
  const sep = parent.includes("\\") && !parent.includes("/") ? "\\" : "/";
  return `${parent.replace(/[\\/]+$/, "")}${sep}${name}`;
}

function sortNodes(a: TreeNode, b: TreeNode): number {
  if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
  return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
}

function findNode(root: TreeNode | null, path: string): TreeNode | null {
  if (!root) return null;
  if (root.path === path) return root;
  if (!root.children) return null;
  for (const c of root.children) {
    const r = findNode(c, path);
    if (r) return r;
  }
  return null;
}

function findParentOf(root: TreeNode | null, path: string): TreeNode | null {
  if (!root || !root.children) return null;
  if (root.children.some((c) => c.path === path)) return root;
  for (const c of root.children) {
    const r = findParentOf(c, path);
    if (r) return r;
  }
  return null;
}

function rewriteChildPaths(node: TreeNode, oldPrefix: string, newPrefix: string) {
  if (!node.children) return;
  for (const c of node.children) {
    if (c.path.startsWith(oldPrefix)) {
      c.path = newPrefix + c.path.slice(oldPrefix.length);
    }
    rewriteChildPaths(c, oldPrefix, newPrefix);
  }
}

export const treeStore = {
  get root(): TreeNode | null {
    return _root;
  },
  set root(v: TreeNode | null) {
    _root = v;
  },
  get selected(): string | null {
    return _selected;
  },
  set selected(p: string | null) {
    _selected = p;
  },
  get renaming(): string | null {
    return _renaming;
  },
  set renaming(p: string | null) {
    _renaming = p;
  },

  /** Insert a new entry into a parent directory (which must already be loaded). */
  insertEntry(parentPath: string, entry: DirEntry) {
    const parent = findNode(_root, parentPath);
    if (!parent || !parent.is_dir) return;
    if (!parent.loaded) return;
    const next: TreeNode = { ...entry, expanded: false, loaded: false };
    parent.children = [...(parent.children ?? []), next].sort(sortNodes);
    parent.expanded = true;
  },

  /** Remove a node from its parent. */
  removeAt(path: string) {
    const parent = findParentOf(_root, path);
    if (!parent || !parent.children) return;
    parent.children = parent.children.filter((c) => c.path !== path);
  },

  /** Rename in-place: updates name, path, and rewrites descendant paths. */
  renameAt(oldPath: string, newName: string): string | null {
    const node = findNode(_root, oldPath);
    if (!node) return null;
    const parent = findParentOf(_root, oldPath);
    const parentPath = parent?.path ?? oldPath.split(/[\\/]/).slice(0, -1).join("/");
    const newPath = joinPath(parentPath, newName);
    if (node.children) rewriteChildPaths(node, node.path, newPath);
    node.path = newPath;
    node.name = newName;
    if (parent?.children) parent.children.sort(sortNodes);
    return newPath;
  },

  /** Move a node into a different directory. dstDirPath must be loaded. */
  moveTo(srcPath: string, dstDirPath: string): string | null {
    const node = findNode(_root, srcPath);
    if (!node) return null;
    const dst = findNode(_root, dstDirPath);
    if (!dst || !dst.is_dir) return null;
    // Remove from old parent
    const oldParent = findParentOf(_root, srcPath);
    if (oldParent?.children) {
      oldParent.children = oldParent.children.filter((c) => c.path !== srcPath);
    }
    const newPath = joinPath(dstDirPath, node.name);
    if (node.children) rewriteChildPaths(node, node.path, newPath);
    node.path = newPath;
    if (dst.loaded) {
      dst.children = [...(dst.children ?? []), node].sort(sortNodes);
      dst.expanded = true;
    }
    return newPath;
  },

  basename,
  joinPath,
};
