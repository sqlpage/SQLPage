// Names the browser bundle relies on at runtime rather than through an import:
// libraries build.rs inlines ahead of our own code, libraries loaded on demand,
// and the objects SQLPage's scripts hang off the window for each other.

/**
 * A library this project ships no type definitions for. Saying `unknown`
 * instead would only move the guesswork to a cast at every call site.
 */
// biome-ignore lint/suspicious/noExplicitAny: that is what an untyped library is
type Untyped = any;

/** Leaflet, loaded from a CDN by sqlpage_map when a page holds a map. */
declare const L: Untyped;

/** ApexCharts, inlined ahead of apexcharts.js by build.rs. */
declare const ApexCharts: Untyped;

/** Tom Select, inlined ahead of tomselect.js by build.rs. */
declare const TomSelect: Untyped;

/** apexcharts.js publishes its initialiser under this name. */
declare var sqlpage_chart: () => void;

/**
 * Tabler's bundled Bootstrap, inlined ahead of sqlpage.js by build.rs. Its
 * widgets are untyped: naming a few of them here would only claim more than
 * this file knows.
 */
interface TablerBootstrap {
  [widget: string]: Untyped;
}

interface Window {
  /** Every chart rendered on the page, in the order they were built. */
  charts?: unknown[];
  tabler: { bootstrap: TablerBootstrap };
  bootstrap?: TablerBootstrap;
}
