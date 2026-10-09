export type InitRoot = Element | Document;

/** Select matching descendants, including the fragment root itself. */
export function select_all<T extends Element>(
  root: InitRoot,
  selector: string,
  element_type: new () => T,
): T[] {
  const descendants = [...root.querySelectorAll(selector)].filter(
    (element): element is T => element instanceof element_type,
  );
  if (root instanceof element_type && root.matches(selector)) {
    descendants.unshift(root);
  }
  return descendants;
}

export function add_init_fn(f: (root: InitRoot) => void) {
  const script = document.currentScript;
  const pending_roots =
    script instanceof HTMLScriptElement ? script.sqlpage_init_roots : undefined;
  const initialize = () => {
    if (script instanceof HTMLScriptElement) delete script.sqlpage_init_roots;
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
