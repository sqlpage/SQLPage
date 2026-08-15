// Names the browser bundles rely on at runtime rather than through an import:
// libraries loaded on demand, and the state SQLPage publishes for page scripts.

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
}
