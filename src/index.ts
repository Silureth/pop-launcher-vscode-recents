#!/usr/bin/env node
import { PopPlugin } from "pop-launcher-toolkit";
import { cleanSearchQuery, getIcon } from "./utils";
import { RecentlyOpened, Vscode } from "./vscode";

/** Must match the `regex` in plugin.ron. */
const TRIGGER = "vc";
const MAX_RESULTS = 8;

class VscodePlugin extends PopPlugin {
  private results: RecentlyOpened = [];

  private constructor(private vscode: Vscode) {
    super();
  }

  static async init(): Promise<VscodePlugin> {
    return new VscodePlugin(await Vscode.init());
  }

  name(): string {
    return "vscode-open-recent";
  }

  search(query: string) {
    const cleaned = cleanSearchQuery(query, TRIGGER);
    this.results = cleaned
      ? this.vscode.search(cleaned)
      : this.vscode.recentlyOpened.slice(0, MAX_RESULTS);

    this.respond_with("Clear");

    if (this.results.length === 0) {
      this.respond_with({
        Append: {
          id: 0,
          name: cleaned ? "No results found." : "No recent VS Code workspaces.",
          description: cleaned ? "Try a different query." : "",
          keywords: null,
          icon: { Name: getIcon("info") },
          exec: null,
          window: null,
        },
      });
    }

    this.results.forEach((entry, idx) => {
      this.respond_with({
        Append: {
          id: idx,
          name: entry.name,
          description: entry.path,
          keywords: null,
          icon: { Name: getIcon(entry.type) },
          exec: null,
          window: null,
        },
      });
    });

    this.respond_with("Finished");
  }

  activate(index: number) {
    const entry = this.results[index];
    if (entry) this.vscode.open(entry);
    this.respond_with("Close");
  }

  exit(): void {
    process.exit(0);
  }
}

VscodePlugin.init().then((plugin) => plugin.run());
