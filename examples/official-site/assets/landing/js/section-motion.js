/** Scroll through long sections before holding their read tail beneath the next cover. */
export function initSectionMotion(root) {
  const sections = [...root.querySelectorAll(".landing-section")];
  const media = matchMedia("(prefers-reduced-motion: reduce)");
  const events = new AbortController();
  const { signal } = events;
  const ease = (value) => {
    const t = Math.max(0, Math.min(1, value));
    return t * t * (3 - 2 * t);
  };
  // Timing follows page geometry; each role's visual pose lives in CSS.
  const timing = {
    heading: { range: 0.3, lead: 150 },
    reading: { range: 0.24, lead: 100 },
    control: { range: 0.23, lead: 110 },
    card: { range: 0.3, lead: 140 },
    detail: { range: 0.26, lead: 120 },
  };
  let layout = [];
  let frame = 0;

  function measure() {
    root.toggleAttribute("data-section-motion", !media.matches);
    // Sticky rectangles describe what is currently painted, not authored flow.
    // The sculpture uses this same temporary switch for its layout measurements.
    root.setAttribute("data-measuring-layout", "");
    const maxScroll = Math.max(
      0,
      document.documentElement.scrollHeight - innerHeight,
    );
    const waveHeights = sections.map(
      (section) =>
        Number.parseFloat(
          getComputedStyle(section).getPropertyValue("--wave-height"),
        ) || 0,
    );
    layout = sections.map((section, index) => {
      const rect = section.getBoundingClientRect();
      const top = rect.top + scrollY;
      // Long sections scroll normally until the whole tail and its wave clearance fit.
      const stickyTop = Math.min(0, innerHeight - rect.height);
      section.style.setProperty("--section-stick-top", `${stickyTop}px`);
      section.style.setProperty("--section-layer", String(index + 1));
      const items = [...section.querySelectorAll("[data-section-reveal]")].map(
        (element) => {
          const bounds = element.getBoundingClientRect();
          const role = element.dataset.sectionReveal;
          const { range, lead } = timing[role];
          const distance = innerHeight * range;
          const across = bounds.left / innerWidth;
          const stagger =
            role === "card" || role === "control" ? across * 55 : 0;
          // Finish in the reading area, before the section holds or the page ends.
          const end = Math.min(
            bounds.top + scrollY - innerHeight * 0.68 + stagger,
            top - stickyTop,
            maxScroll,
          );
          // A trailing card must finish its entrance before the next wave can
          // begin its exit. Reserve half of the clear gap for undimmed reading.
          const trailingGap =
            rect.bottom - bounds.bottom - (waveHeights[index + 1] || 0) * 0.75;
          const exitLead = Math.max(
            0,
            Math.min(lead, innerHeight * 0.12, trailingGap * 0.5),
          );
          const direction =
            (role === "card" || role === "detail") &&
            bounds.left + bounds.width / 2 < innerWidth / 2
              ? -1
              : 1;
          element.style.setProperty("--reveal-direction", String(direction));
          return {
            element,
            exitLead,
            top: bounds.top + scrollY - top,
            height: bounds.height,
            start: end - distance,
            distance,
          };
        },
      );
      return { section, top, items, waveHeight: waveHeights[index] };
    });
    root.removeAttribute("data-measuring-layout");
    update();
  }

  function update() {
    frame = 0;
    if (media.matches) return;
    // Read before writing: no per-element layout work during scrolling.
    const rectangles = layout.map(({ section }) =>
      section.getBoundingClientRect(),
    );
    layout.forEach(({ section, items }, index) => {
      const next = rectangles[index + 1];
      const cover = next ? ease((innerHeight - next.top) / innerHeight) : 0;
      section.style.setProperty("--section-cover", String(cover));
      for (const item of items) {
        // The opening already choreographs the demo; only give it an exit.
        const entrance =
          index === 0 ? 1 : ease((scrollY - item.start) / item.distance);
        const itemTop = rectangles[index].top + item.top;
        // Begin receding before the wave touches the element's bottom. Finish
        // near its top as it is covered, rather than after it is already hidden.
        const front = next
          ? next.top - layout[index + 1].waveHeight * 0.75
          : Infinity;
        const end = itemTop + Math.min(item.height * 0.4, 100);
        const distance = item.height - (end - itemTop) + item.exitLead;
        // Give scrolling content an outro at the top; held tails recede under the wave.
        // Both begin only after the item has had room to be read in full.
        const exit = Math.min(
          ease((front - end) / distance),
          ease(
            (itemTop + item.height) /
              Math.max(1, Math.min(innerHeight * 0.24, item.height * 0.6)),
          ),
        );
        item.element.style.setProperty("--reveal-enter", String(entrance));
        item.element.style.setProperty("--reveal-exit", String(exit));
      }
    });
  }

  function schedule() {
    if (!frame) frame = requestAnimationFrame(update);
  }

  // Native focus scrolling cannot uncover an earlier sticky sheet or reach its
  // offscreen upper content on a tall section. Recover its
  // ordinary document position for keyboard users instead of hiding controls
  // from the tab order or lifting a focused sheet over the later sections.
  root.addEventListener(
    "focusin",
    (event) => {
      // Pointer focus must not move a control between mouse-down and click.
      if (media.matches || !event.target.matches(":focus-visible")) return;
      const index = layout.findIndex(({ section }) =>
        section.contains(event.target),
      );
      if (index < 0) return;
      const current = layout[index];
      const bounds = event.target.getBoundingClientRect();
      const next = layout[index + 1]?.section.getBoundingClientRect();
      const outsideViewport = bounds.top < 0 || bounds.bottom > innerHeight;
      const covered = next && bounds.bottom > next.top;
      if (outsideViewport || covered) {
        const relativeTop =
          bounds.top - current.section.getBoundingClientRect().top;
        window.scrollTo({
          top: current.top + relativeTop - innerHeight * 0.35,
          behavior: "instant",
        });
      }
      schedule();
    },
    { signal },
  );
  window.addEventListener("scroll", schedule, { passive: true, signal });
  window.addEventListener("resize", measure, { signal });
  media.addEventListener("change", measure, { signal });
  const observer = new ResizeObserver(measure);
  observer.observe(root);
  for (const section of sections) observer.observe(section);
  measure();
  return () => {
    events.abort();
    observer.disconnect();
    cancelAnimationFrame(frame);
    root.removeAttribute("data-section-motion");
  };
}
