export function add_init_fn(f: () => void) {
  document.addEventListener("DOMContentLoaded", f);
  document.addEventListener("fragment-loaded", f);
  if (document.readyState !== "loading") setTimeout(f, 0);
}
