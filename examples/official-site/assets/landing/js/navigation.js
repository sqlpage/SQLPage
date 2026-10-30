/** Native dialog owns focus trapping, Escape, and focus restoration. */
export function initNavigation(root) {
  const dialog = root.querySelector("#navigation");
  const trigger = root.querySelector(".mobile-menu");
  const events = new AbortController();
  const { signal } = events;
  // Keep link destinations authored once, in the accessible desktop nav.
  const mobileLinks = [...root.querySelectorAll(".desktop-nav a")].map(
    (link) => {
      const copy = link.cloneNode(true);
      copy.removeAttribute("class");
      return copy;
    },
  );
  dialog.querySelector("nav").replaceChildren(...mobileLinks);
  trigger.hidden = false;
  trigger.addEventListener(
    "click",
    () => {
      dialog.showModal();
      trigger.setAttribute("aria-expanded", "true");
    },
    { signal },
  );
  dialog.addEventListener(
    "close",
    () => trigger.setAttribute("aria-expanded", "false"),
    { signal },
  );
  dialog.addEventListener(
    "click",
    (event) => {
      const bounds = dialog.getBoundingClientRect();
      if (
        event.target === dialog &&
        (event.clientX < bounds.left ||
          event.clientX > bounds.right ||
          event.clientY < bounds.top ||
          event.clientY > bounds.bottom)
      )
        dialog.close();
      if (event.target.closest("a")) dialog.close();
    },
    { signal },
  );
  const desktop = matchMedia("(min-width: 760px)");
  desktop.addEventListener(
    "change",
    () => {
      if (desktop.matches && dialog.open) dialog.close();
    },
    { signal },
  );
  for (const button of root.querySelectorAll(".glass-button")) {
    button.addEventListener(
      "pointermove",
      (event) => {
        const bounds = button.getBoundingClientRect();
        button.style.setProperty(
          "--glass-x",
          `${((event.clientX - bounds.left) / bounds.width) * 100}%`,
        );
        button.style.setProperty(
          "--glass-y",
          `${((event.clientY - bounds.top) / bounds.height) * 100}%`,
        );
      },
      { signal },
    );
    button.addEventListener(
      "pointerleave",
      () => {
        button.style.removeProperty("--glass-x");
        button.style.removeProperty("--glass-y");
      },
      { signal },
    );
  }
  return () => {
    events.abort();
    dialog.close();
    trigger.hidden = true;
  };
}
