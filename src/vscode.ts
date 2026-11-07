import { spawn } from "child_process";
import { existsSync, readFileSync } from "fs";
import { readdir, readFile, stat } from "fs/promises";
import { homedir } from "os";
import { basename, dirname, join } from "path";
import { fileURLToPath } from "url";
import Fuse from "fuse.js";

const EXECUTABLE = "code";
const WORKSPACE_STORAGE = join(homedir(), ".config/Code/User/workspaceStorage");
const CONFIG_FILE = join(__dirname, "..", "config.json");
const MAX_RESULTS = 8;
const FUSE_OPTIONS = {
  keys: ["name", "path"],
};

interface Config {
  always_open_in_new_window: boolean;
  hide_deleted_entries: boolean;
  "vscode-cli-additional-args": string[];
}

const DEFAULT_CONFIG: Config = {
  always_open_in_new_window: true,
  hide_deleted_entries: true,
  "vscode-cli-additional-args": [],
};

interface WorkspaceJson {
  folder?: string;
  workspace?: string;
}

export interface Entry {
  type: "workspace" | "folder";
  name: string;
  /** Parent directory shown as the description (home shortened to ~). */
  path: string;
  /** Original VS Code URI (file:// or vscode-remote://). */
  uri: string;
  /** Local filesystem path, or null for remote URIs. */
  localPath: string | null;
}

export type RecentlyOpened = Entry[];

export class Vscode {
  recentlyOpened: RecentlyOpened = [];
  private config: Config = loadConfig();

  private constructor() {}

  static async init() {
    const vscode = new Vscode();
    vscode.recentlyOpened = await vscode.fetchRecentlyOpened();
    return vscode;
  }

  /**
   * VS Code 1.120+ no longer keeps `history.recentlyOpenedPathsList` in the
   * global state DB, so recents are reconstructed from workspaceStorage:
   * each subfolder has a workspace.json naming the folder/workspace, and the
   * mtime of its state.vscdb tells when it was last used.
   */
  private async fetchRecentlyOpened(): Promise<RecentlyOpened> {
    let dirs: string[];
    try {
      dirs = await readdir(WORKSPACE_STORAGE);
    } catch {
      return [];
    }

    const found = await Promise.all(
      dirs.map(async (dir) => {
        try {
          const [content, dbStat] = await Promise.all([
            readFile(join(WORKSPACE_STORAGE, dir, "workspace.json"), "utf-8"),
            stat(join(WORKSPACE_STORAGE, dir, "state.vscdb")),
          ]);
          const data = JSON.parse(content) as WorkspaceJson;
          const entry = toEntry(data);
          return entry ? { entry, mtime: dbStat.mtimeMs } : null;
        } catch {
          return null;
        }
      })
    );

    const seen = new Set<string>();
    return found
      .filter((f): f is { entry: Entry; mtime: number } => f !== null)
      .sort((a, b) => b.mtime - a.mtime)
      .map((f) => f.entry)
      .filter((entry) => {
        if (seen.has(entry.uri)) return false;
        seen.add(entry.uri);
        if (this.config.hide_deleted_entries && entry.localPath) {
          return existsSync(entry.localPath);
        }
        return true;
      });
  }

  search(query: string): RecentlyOpened {
    const fuse = new Fuse(this.recentlyOpened, FUSE_OPTIONS);
    return fuse
      .search(query)
      .slice(0, MAX_RESULTS)
      .map((result) => result.item);
  }

  open(entry: Entry) {
    const args: string[] = [
      this.config.always_open_in_new_window ? "--new-window" : "--reuse-window",
      ...this.config["vscode-cli-additional-args"],
    ];

    if (entry.localPath) {
      args.push(entry.localPath);
    } else {
      args.push(entry.type === "workspace" ? "--file-uri" : "--folder-uri", entry.uri);
    }

    const child = spawn(EXECUTABLE, args, { detached: true, stdio: "ignore" });
    child.on("error", () => {}); // e.g. `code` not in PATH; nothing useful to report to the launcher
    child.unref();
  }
}

function loadConfig(): Config {
  try {
    return { ...DEFAULT_CONFIG, ...JSON.parse(readFileSync(CONFIG_FILE, "utf-8")) };
  } catch {
    return DEFAULT_CONFIG;
  }
}

function toEntry(data: WorkspaceJson): Entry | null {
  const uri = data.workspace || data.folder;
  if (!uri) return null;

  let localPath: string | null = null;
  let displayPath: string;
  if (uri.startsWith("file://")) {
    localPath = fileURLToPath(uri);
    displayPath = localPath;
  } else {
    displayPath = decodeURIComponent(uri);
  }

  const home = homedir();
  const parent = dirname(displayPath);
  return {
    type: data.workspace ? "workspace" : "folder",
    name: basename(displayPath).replace(/\.code-workspace$/, ""),
    path: parent === home || parent.startsWith(home + "/") ? "~" + parent.slice(home.length) : parent,
    uri,
    localPath,
  };
}
