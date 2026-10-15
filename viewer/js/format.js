// Shared formatting helpers: one copy of each small formatter used across
// viewer modules, so a fix in one place cannot miss the others.

/** The message of a caught error: Error message or String(value). */
export function errorMessage(err) {
  return err instanceof Error ? err.message : String(err);
}

/** Human size: bytes below 1 KiB, KB below 1 MiB, MB above (one decimal). */
export function formatSize(size) {
  if (size < 1024) return `${size} bytes`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

/** The SNIP levels every diagnostics view groups by. */
export const SNIP_LEVELS = Object.freeze(['SNIP 1', 'SNIP 2', 'SNIP 3']);
