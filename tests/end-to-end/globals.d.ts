// What the browser tests reach for on the page: widgets that the scripts under
// test attach to elements at runtime, and libraries the page loads itself.

interface TomSelectInstance {
  getValue(): string | string[];
  setTextboxValue(value: string): void;
  focus(): void;
  options: Record<string, { label?: string } | undefined>;
}

interface HTMLElement {
  /** Attached by sqlpage_select_dropdown to every select it takes over. */
  tomselect?: TomSelectInstance;
}

interface Window {
  /** Tabler's bundled Bootstrap, inlined ahead of sqlpage.js by build.rs. */
  tabler?: {
    bootstrap: { Toast: { getInstance(element: Element): unknown } };
  };
}
