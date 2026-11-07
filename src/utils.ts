import { join } from "path";

// The bundle lives in <plugin>/dist/index.cjs, so assets are one level up.
const ASSETS_DIR = join(__dirname, "..", "assets");

const ICONS: Record<string, string> = {
  file: "file.svg",
  workspace: "workspace.svg",
  folder: "folder.svg",
  info: "info.svg",
  error: "error.svg",
  warning: "warning.svg",
};

/** Strips the leading trigger word (e.g. "vc") and surrounding whitespace. */
export function cleanSearchQuery(query: string, trigger: string): string {
  return query.trim().replace(new RegExp(`^${trigger}\\s*`), "").trim();
}

export function getIcon(type: string): string {
  const file = ICONS[type];
  return file ? join(ASSETS_DIR, file) : "";
}
