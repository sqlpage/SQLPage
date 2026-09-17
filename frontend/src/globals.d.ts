// Names the browser bundle relies on at runtime rather than through an import:
// libraries loaded on demand, and what a page may provide for our scripts.

/**
 * A library this project ships no type definitions for. Saying `unknown`
 * instead would only move the guesswork to a cast at every call site.
 */
// biome-ignore lint/suspicious/noExplicitAny: that is what an untyped library is
type Untyped = any;

/** Leaflet, loaded from a CDN by sqlpage_map when a page holds a map. */
declare const L: Untyped;

interface Window {
  /** Every chart rendered on the page, in the order they were built. */
  charts?: unknown[];
  /** A Bootstrap a page loaded for itself, preferred over the bundled copy. */
  bootstrap?: typeof import("@tabler/core").bootstrap;
}
