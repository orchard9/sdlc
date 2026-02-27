import {
  readFileSync,
  writeFileSync,
  unlinkSync,
  existsSync,
  mkdirSync,
} from "node:fs";
import { dirname, resolve } from "node:path";

function sessionPath(cwd: string, slug: string): string {
  return resolve(cwd, `.sdlc/features/${slug}/.agent-session`);
}

/**
 * Load a previously saved Claude Agent SDK session ID for a feature.
 * Returns undefined if no session is saved.
 */
export function loadSession(cwd: string, slug: string): string | undefined {
  const path = sessionPath(cwd, slug);
  if (!existsSync(path)) return undefined;
  try {
    const value = readFileSync(path, "utf8").trim();
    return value.length > 0 ? value : undefined;
  } catch {
    return undefined;
  }
}

/**
 * Persist a Claude Agent SDK session ID for a feature so future runs
 * can resume with full conversation context.
 */
export function saveSession(cwd: string, slug: string, sessionId: string): void {
  const path = sessionPath(cwd, slug);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, sessionId, "utf8");
}

/**
 * Remove the saved session for a feature, forcing a fresh session on the
 * next run. Call this when the feature is archived, released, or when
 * the session is known to be invalid.
 */
export function clearSession(cwd: string, slug: string): void {
  const path = sessionPath(cwd, slug);
  try {
    unlinkSync(path);
  } catch {
    // File didn't exist — that's fine
  }
}
