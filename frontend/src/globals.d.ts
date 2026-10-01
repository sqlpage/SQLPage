// Names the browser bundle relies on at runtime rather than through an import.

interface Window {
  /** Every chart rendered on the page, in the order they were built. */
  charts?: unknown[];
  /** A Bootstrap a page loaded for itself, preferred over the bundled copy. */
  bootstrap?: typeof import("@tabler/core").bootstrap;
}
