const search = document.querySelector(".docs-search input");
const shortcut = document.querySelector(".docs-search kbd");
shortcut.textContent = /Mac|iPhone|iPad/.test(navigator.platform)
  ? "⌘ K"
  : "Ctrl K";
shortcut.hidden = false;
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
  if (
    event.key.toLowerCase() === "k" &&
    (event.ctrlKey || event.metaKey) &&
    !event.altKey &&
    !event.shiftKey &&
    !event.target.closest(
      "input, textarea, select, [contenteditable], [role=textbox], dialog, [aria-modal=true]",
    )
  ) {
    event.preventDefault();
    shortcutOrigin = document.activeElement;
    search.focus();
    search.select();
  }
});

// Keep navigation directly browsable when JavaScript is unavailable.
const narrow = window.matchMedia("(max-width: 61.999rem)");
const rails = document.querySelectorAll(".docs-disclosure");
function sizeRails() {
  for (const rail of rails) rail.open = !narrow.matches;
}
sizeRails();
narrow.addEventListener("change", sizeRails);

const filter = document.querySelector(".docs-filter");
if (filter) {
  const input = filter.querySelector("input");
  const list = document.getElementById(input.getAttribute("aria-controls"));
  const links = Array.from(list.querySelectorAll("a"));
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
    list.scrollTop = 0;
    if (!query) revealCurrent();
  });

  const current = list.querySelector('[aria-current="page"]');
  function revealCurrent() {
    if (!current || current.hidden || !list.clientHeight) return;
    const top = current.offsetTop;
    if (
      top < list.scrollTop ||
      top + current.offsetHeight > list.scrollTop + list.clientHeight
    ) {
      list.scrollTop = top - (list.clientHeight - current.offsetHeight) / 2;
    }
  }
  revealCurrent();
  list.closest("details").addEventListener("toggle", revealCurrent);
}

const sections = Array.from(document.querySelectorAll(".docs-outline a"))
  .map((link) => ({
    link,
    target: document.getElementById(link.hash.slice(1)),
  }))
  .filter(({ target }) => target);
if (sections.length) {
  const header = document.querySelector(".docs-header");
  let activeLink;
  function markSection() {
    const top = header.getBoundingClientRect().bottom + 32;
    // A fragment can land below the header when fonts or a preview finish loading.
    const selected = sections.find(({ link, target }) => {
      if (link.hash !== location.hash) return false;
      const position = target.getBoundingClientRect().top;
      return position >= 0 && position < innerHeight / 2;
    });
    const nearEnd =
      window.scrollY > 0 &&
      window.scrollY + innerHeight >= document.documentElement.scrollHeight - 1;
    const section =
      selected ??
      (nearEnd
        ? sections.at(-1)
        : (sections.findLast(
            ({ target }) => target.getBoundingClientRect().top <= top,
          ) ?? sections[0]));
    const { link } = section;
    if (activeLink === link) return;
    activeLink?.removeAttribute("aria-current");
    link.setAttribute("aria-current", "location");
    activeLink = link;
  }
  new ResizeObserver(markSection).observe(document.querySelector("main"));
  markSection();
  window.addEventListener("scroll", markSection, { passive: true });
  window.addEventListener("resize", markSection);
  window.addEventListener("hashchange", markSection);
}
