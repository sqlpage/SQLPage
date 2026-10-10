import { initComponentDemos } from "./component-demos.js";

/** Accessible tab groups share selection and keyboard behavior. */
function tabs(group, attribute, select, signal) {
  const buttons = [...group.querySelectorAll("button")];
  function activate(button) {
    for (const item of buttons) {
      item.setAttribute(attribute, String(item === button));
      item.tabIndex = item === button ? 0 : -1;
    }
    select(button);
  }
  group.addEventListener(
    "click",
    (event) => {
      const button = event.target.closest("button");
      if (buttons.includes(button)) activate(button);
    },
    { signal },
  );
  group.addEventListener(
    "keydown",
    (event) => {
      const visible = buttons.filter((button) => !button.hidden);
      const index = visible.indexOf(document.activeElement);
      if (index < 0) return;
      let next;
      if (["ArrowRight", "ArrowDown"].includes(event.key))
        next = (index + 1) % visible.length;
      if (["ArrowLeft", "ArrowUp"].includes(event.key))
        next = (index + visible.length - 1) % visible.length;
      if (event.key === "Home") next = 0;
      if (event.key === "End") next = visible.length - 1;
      if (next !== undefined) {
        event.preventDefault();
        activate(visible[next]);
        visible[next].focus();
      }
    },
    { signal },
  );
  return activate;
}

export function initSections(root) {
  const events = new AbortController();
  const { signal } = events;
  const cleanupDemos = initComponentDemos(root, tabs, signal);
  tabs(
    root.querySelector('[aria-label="Application stack"]'),
    "aria-pressed",
    (button) => {
      const comparison = root.querySelector(".stack-comparison");
      comparison.dataset.stack = button.dataset.stack;
      for (const copy of comparison.querySelectorAll("[data-stack-copy]"))
        copy.setAttribute(
          "aria-hidden",
          String(copy.dataset.stackCopy !== button.dataset.stack),
        );
      root.querySelector("[data-stack-caption]").textContent =
        button.dataset.stack === "sqlpage"
          ? "-- Same five layers. SQLPage handles the plumbing."
          : "-- 5 layers, all yours.";
    },
    signal,
  );
  tabs(
    root.querySelector('[aria-label="Frontend features"]'),
    "aria-selected",
    (button) => {
      for (const id of ["customize", "ship", "instant", "safe"])
        root.querySelector(`#${id}-panel`).hidden =
          id !== button.dataset.feature;
    },
    signal,
  );
  return () => {
    events.abort();
    cleanupDemos();
  };
}
