import { initExperience } from "./experience.js";
import { initNavigation } from "./navigation.js";

const root = document.querySelector(".sqlpage-world");
// Author one sequence; its decorative copy never enters the tab order.
const sequence = root.querySelector(".ribbon-sequence");
const copy = sequence.cloneNode(true);
copy.setAttribute("aria-hidden", "true");
for (const link of copy.querySelectorAll("a")) link.tabIndex = -1;
sequence.after(copy);
root.dataset.enhanced = "true";
const cleanups = [initNavigation(root), initExperience(root)];
window.addEventListener("pagehide", (event) => {
  // Preserve state when entering the browser's back/forward cache.
  if (!event.persisted)
    cleanups.forEach((cleanup) => {
      cleanup();
    });
});
