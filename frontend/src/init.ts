export type InitRoot = Element | Document;

// The script element carries roots across independently bundled initializers.
export type InitScript = HTMLScriptElement & {
  sqlpage_init_roots?: Set<InitRoot>;
};

/** Select matching descendants, including the fragment root itself. */
export function select_all<T extends Element>(
  root: InitRoot,
  selector: string,
): T[] {
  const descendants = [...root.querySelectorAll<T>(selector)];
  if (root instanceof Element && root.matches(selector)) {
    descendants.unshift(root as T);
  }
  return descendants;
}

export function add_init_fn(f: (root: InitRoot) => void) {
  const script = document.currentScript as InitScript | null;
  const pending_roots = script?.sqlpage_init_roots;
  const initialize = () => {
    if (script) delete script.sqlpage_init_roots;
    for (const root of pending_roots ?? [document]) {
      if (root instanceof Document || root.isConnected) f(root);
    }
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initialize, { once: true });
  } else {
    setTimeout(initialize, 0);
  }
  document.addEventListener("fragment-loaded", ({ target }) => {
    if (target instanceof Element || target instanceof Document) f(target);
  });
}
