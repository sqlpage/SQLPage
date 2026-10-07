export type InitRoot = Element | Document;

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
  document.addEventListener("DOMContentLoaded", () => f(document));
  document.addEventListener("fragment-loaded", ({ target }) => {
    if (target instanceof Element || target instanceof Document) f(target);
  });
  if (document.readyState !== "loading") setTimeout(() => f(document), 0);
}
