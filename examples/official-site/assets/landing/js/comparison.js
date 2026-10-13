/** Native scrolling remains available without enhancement, including on touch screens. */
export function initComparison(root, signal) {
  const section = root.querySelector(".comparison-section");
  const track = section.querySelector(".comparison-track");
  const cards = [...track.children];
  const previous = section.querySelector("[data-comparison-previous]");
  const next = section.querySelector("[data-comparison-next]");
  const status = section.querySelector("[data-comparison-status]");
  const motion = matchMedia("(prefers-reduced-motion: reduce)");

  function positions() {
    // Layout offsets stay accurate while section reveals scale the painted track.
    const origin = cards[0].offsetLeft;
    return cards.map((card) => card.offsetLeft - origin);
  }

  function update() {
    const offsets = positions();
    const first = offsets.findIndex(
      (left, index) => left + cards[index].offsetWidth > track.scrollLeft + 2,
    );
    const last = offsets.findLastIndex(
      (left) => left < track.scrollLeft + track.clientWidth - 2,
    );
    previous.disabled = track.scrollLeft <= 2;
    next.disabled =
      track.scrollLeft >= track.scrollWidth - track.clientWidth - 2;
    const range =
      first === last ? String(first + 1) : `${first + 1}–${last + 1}`;
    const label = `${range} of ${cards.length} alternatives`;
    // Avoid repeating live announcements during every frame of a smooth scroll.
    if (status.textContent !== label) status.textContent = label;
  }

  function goTo(left) {
    track.scrollTo({
      left,
      behavior: motion.matches ? "instant" : "smooth",
    });
  }

  function move(direction) {
    const offsets = positions();
    const target =
      direction > 0
        ? offsets.find((left) => left > track.scrollLeft + 2)
        : offsets.findLast((left) => left < track.scrollLeft - 2);
    goTo(target ?? (direction > 0 ? track.scrollWidth : 0));
  }

  previous.addEventListener("click", () => move(-1), { signal });
  next.addEventListener("click", () => move(1), { signal });
  track.addEventListener("scroll", update, { passive: true, signal });
  track.addEventListener(
    "keydown",
    (event) => {
      if (event.target !== track) return;
      switch (event.key) {
        case "ArrowLeft":
          move(-1);
          break;
        case "ArrowRight":
          move(1);
          break;
        case "Home":
          goTo(0);
          break;
        case "End":
          goTo(track.scrollWidth);
          break;
        default:
          return;
      }
      event.preventDefault();
    },
    { signal },
  );
  const observer = new ResizeObserver(update);
  observer.observe(track);
  section.querySelector(".comparison-controls").hidden = false;
  update();
  return () => observer.disconnect();
}
