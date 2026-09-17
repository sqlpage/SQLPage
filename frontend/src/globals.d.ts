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

/**
 * A Bootstrap a page loaded for itself, preferred over the bundled copy. Its
 * widgets are untyped: naming a few of them here would only claim more than
 * this file knows.
 */
interface PageBootstrap {
  [widget: string]: Untyped;
}

interface Window {
  /** Every chart rendered on the page, in the order they were built. */
  charts?: unknown[];
  bootstrap?: PageBootstrap;
}
