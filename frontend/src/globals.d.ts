// What the bundle installs on the page at runtime rather than through an
// import: the page's own scripts and the browser tests read these back.

interface Window {
  /** Every chart rendered on the page, in the order they were built. */
  charts?: import("./apexcharts.ts").RenderedChart[];
  /** A Bootstrap a page loaded for itself, preferred over the bundled copy. */
  bootstrap?: typeof import("@tabler/core").bootstrap;
}

interface HTMLElement {
  /** Attached by sqlpage_select_dropdown to every select it takes over. */
  tomselect?: import("tom-select/popular").default;
}
