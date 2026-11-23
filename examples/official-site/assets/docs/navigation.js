const search = document.getElementById("docs-search-input");
const shortcut = document.querySelector(".docs-search kbd");
if (shortcut) {
  shortcut.textContent = /Mac|iPhone|iPad/.test(navigator.platform)
    ? "⌘ K"
    : "Ctrl K";
  shortcut.hidden = false;
}
search.setAttribute("aria-keyshortcuts", "Control+k Meta+k");
let shortcutOrigin;

search.addEventListener("blur", () => {
  shortcutOrigin = null;
});
document.addEventListener("keydown", (event) => {
  if (event.defaultPrevented || event.isComposing) return;
  if (event.key === "Escape" && event.target === search && shortcutOrigin) {
    event.preventDefault();
    const origin = shortcutOrigin;
    search.blur();
    origin.focus({ preventScroll: true });
    return;
  }
  if (event.key.toLowerCase() !== "k" || !(event.ctrlKey || event.metaKey))
    return;
  if (
    event.altKey ||
    event.shiftKey ||
    event.target.closest(
      "input, textarea, select, [contenteditable], [role=textbox], dialog, [aria-modal=true]",
    )
  )
    return;
  event.preventDefault();
  shortcutOrigin = document.activeElement;
  search.focus();
  search.select();
});

// Keep navigation directly browsable when JavaScript is unavailable.
const desktop = window.matchMedia("(min-width: 62rem)");
const rails = document.querySelectorAll(".docs-disclosure");
function sizeRails() {
  for (const rail of rails) rail.open = desktop.matches;
}
sizeRails();
desktop.addEventListener("change", sizeRails);

const filter = document.querySelector(".docs-filter");
if (filter) {
  const input = filter.querySelector("input");
  const list = document.getElementById(input.getAttribute("aria-controls"));
  const links = list.querySelectorAll("a");
  const status = filter.parentElement.querySelector('[role="status"]');
  filter.hidden = false;
  input.addEventListener("input", () => {
    const query = input.value.trim().toLowerCase();
    let matches = 0;
    for (const link of links) {
      link.hidden = !link.textContent.toLowerCase().includes(query);
      if (!link.hidden) matches++;
    }
    status.textContent = query ? `${matches} of ${links.length} shown` : "";
    if (query) list.scrollTop = 0;
    else revealCurrent();
  });

  const current = list.querySelector('[aria-current="page"]');
  function revealCurrent() {
    // Scroll only the sidebar. The document may have opened at a fragment.
    if (current)
      list.scrollTop =
        current.offsetTop - (list.clientHeight - current.offsetHeight) / 2;
  }
  revealCurrent();
  list.closest("details").addEventListener("toggle", revealCurrent);
}

const sections = Array.from(
  document.querySelectorAll("#docs-outline a"),
  (link) => ({
    link,
    target: document.getElementById(link.hash.slice(1)),
  }),
).filter(({ target }) => target);
if (sections.length) {
  const main = document.querySelector("main");
  let current;
  function markSection() {
    const atEnd =
      scrollY > 0 &&
      scrollY + innerHeight >= document.documentElement.scrollHeight - 1;
    // The browser places fragment targets at this same offset below the sticky header.
    const top = parseFloat(getComputedStyle(main).scrollMarginTop) + 1;
    const { link } =
      (atEnd
        ? sections.at(-1)
        : sections.findLast(
            ({ target }) => target.getBoundingClientRect().top <= top,
          )) ?? sections[0];
    if (link === current) return;
    current?.removeAttribute("aria-current");
    link.setAttribute("aria-current", "location");
    current = link;
  }
  new ResizeObserver(markSection).observe(main);
  window.addEventListener("scroll", markSection, { passive: true });
  window.addEventListener("resize", markSection);
}
