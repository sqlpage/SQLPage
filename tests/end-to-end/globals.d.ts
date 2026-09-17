// What the browser tests reach for on the page: widgets that the scripts under
// test attach to elements at runtime.

interface HTMLElement {
  /** Attached by sqlpage_select_dropdown to every select it takes over. */
  tomselect?: import("tom-select/popular").default;
}
