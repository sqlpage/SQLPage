/** Shared selection and roving focus for the landing page’s tab groups. */
export function selectTab(group, button, attribute = "aria-selected") {
  for (const item of group.querySelectorAll("button")) {
    item.setAttribute(attribute, String(item === button));
    item.tabIndex = item === button ? 0 : -1;
  }
}

export function initTabs(group, attribute, select, signal) {
  const buttons = [...group.querySelectorAll("button")];
  function activate(button) {
    selectTab(group, button, attribute);
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
}
