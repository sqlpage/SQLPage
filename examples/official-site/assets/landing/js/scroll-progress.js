import { layoutLandingFrame, SCULPTURE_FRAME } from "./scene/landing-frame.js";

/** One sculpture, measured document anchors, and ordinary page scrolling. */
export function initScrollProgress(section, mount, state) {
  const root = section.closest(".sqlpage-world");
  const preview = root.querySelector(".scene-preview");
  const stops = [...root.querySelectorAll("[data-scene-stop]")];
  const events = new AbortController();
  let frame = 0;
  let anchors = [];
  function measure() {
    const hero = layoutLandingFrame(
      mount,
      root.querySelector(".anchor-letter"),
    );
    anchors = [{ scroll: 0, ...hero }];
    for (const stop of stops) {
      const rect = stop.getBoundingClientRect();
      // The visible sculpture occupies these bounds in its square reference frame.
      const size = rect.width / SCULPTURE_FRAME.bodyWidth;
      anchors.push({
        scroll: stop.closest("section").offsetTop - window.innerHeight * 0.12,
        left: rect.left - size * SCULPTURE_FRAME.bounds.left,
        top: rect.top + window.scrollY - size * SCULPTURE_FRAME.bounds.top,
        size,
      });
    }
    update();
  }
  function update() {
    frame = 0;
    const scroll = window.scrollY;
    let index = 0;
    while (index < anchors.length - 2 && scroll > anchors[index + 1].scroll)
      index++;
    const a = anchors[index];
    const b = anchors[index + 1];
    // Hold each composition while its content is read. Travel near the next
    // section boundary so the sculpture never drifts over the live controls.
    const start =
      b.scroll - Math.min(innerHeight * 0.75, (b.scroll - a.scroll) * 0.55);
    const fraction = Math.max(
      0,
      Math.min(1, (scroll - start) / (b.scroll - start)),
    );
    const t = fraction * fraction * (3 - 2 * fraction);
    state.frame = {
      left: a.left + (b.left - a.left) * t,
      top: a.top + (b.top - a.top) * t - scroll,
      size: a.size + (b.size - a.size) * t,
    };
    Object.assign(preview.style, {
      left: `${state.frame.left}px`,
      top: `${state.frame.top}px`,
      width: `${state.frame.size}px`,
      height: `${state.frame.size}px`,
    });
  }
  const schedule = () => {
    if (!frame) frame = requestAnimationFrame(update);
  };
  window.addEventListener("scroll", schedule, {
    passive: true,
    signal: events.signal,
  });
  window.addEventListener("resize", measure, { signal: events.signal });
  // Layout changes include font loading and the responsive hero's measured height.
  const observer = new ResizeObserver(measure);
  observer.observe(root);
  measure();
  return () => {
    events.abort();
    observer.disconnect();
    cancelAnimationFrame(frame);
  };
}
