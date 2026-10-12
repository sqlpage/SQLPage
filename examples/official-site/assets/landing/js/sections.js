/** Wire section controls; motion and live-preview loading live in separate modules. */
import { initComponentDemos } from "./component-demos.js";
import { initTabs } from "./tabs.js";

export function initSections(root) {
  const events = new AbortController();
  const { signal } = events;
  const cleanupDemos = initComponentDemos(root, signal);
  initTabs(
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
  initTabs(
    root.querySelector('[aria-label="Frontend features"]'),
    "aria-selected",
    (button) => {
      for (const panel of root.querySelectorAll(".frontend-panel"))
        panel.hidden = panel.id !== button.getAttribute("aria-controls");
    },
    signal,
  );
  return () => {
    events.abort();
    cleanupDemos();
  };
}
