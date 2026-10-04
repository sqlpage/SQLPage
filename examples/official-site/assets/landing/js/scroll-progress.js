import { layoutLandingFrame, SCULPTURE_FRAME } from "./scene/landing-frame.js";

/** One sculpture, measured document anchors, and ordinary page scrolling. */
export function initScrollProgress(section, mount, state) {
  const root = section.closest(".sqlpage-world");
  const preview = root.querySelector(".scene-preview");
  const stops = [...root.querySelectorAll("[data-scene-stop]")];
  const events = new AbortController();
  let frame = 0;
  let anchors = [];
  let heroRange = 0;
  const clamp = (value) => Math.max(0, Math.min(1, value));
  const ease = (value) => {
    const t = clamp(value);
    return t * t * (3 - 2 * t);
  };
  const mix = (a, b, t) => ({
    left: a.left + (b.left - a.left) * t,
    top: a.top + (b.top - a.top) * t,
    size: a.size + (b.size - a.size) * t,
  });
  const viewport = section.querySelector(".viewport");
  const intro = section.querySelector(".intro-row");
  const header = section.querySelector(".site-header");
  const ribbon = section.querySelector(".sql-ribbon");
  function measure() {
    // Measure the authored hero pose, independent of its current text fade/shift.
    section.style.setProperty("--progress", "0");
    section.style.setProperty("--intro-shift", "0px");
    const hero = layoutLandingFrame(
      mount,
      root.querySelector(".anchor-letter"),
    );
    anchors = [{ scroll: 0, ...hero }];
    heroRange = Math.max(0, section.offsetHeight - viewport.offsetHeight);
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
    const p = heroRange
      ? clamp(
          (scroll - Math.max(0, viewport.offsetHeight - innerHeight)) /
            heroRange,
        )
      : 0;
    const opacity = 1 - ease((p - 0.08) / 0.6);
    section.style.setProperty("--progress", String(p));
    section.style.setProperty("--intro-opacity", String(opacity));
    section.style.setProperty("--intro-shift", `${-p * 90}px`);
    section.style.setProperty("--ribbon-opacity", String(1 - ease(p / 0.5)));
    intro.inert = header.inert = opacity < 0.05;
    ribbon.inert = p > 0.5;
    state.turn = 0;
    if (heroRange && scroll <= anchors[1].scroll) {
      const bounds = SCULPTURE_FRAME.bounds;
      const size = Math.min(
        (innerWidth * 0.9) / SCULPTURE_FRAME.bodyWidth,
        (innerHeight * 0.93) / (bounds.bottom - bounds.top),
      );
      const gallery = {
        left:
          innerWidth / 2 - size * (bounds.left + SCULPTURE_FRAME.bodyWidth / 2),
        top: innerHeight / 2 - (size * (bounds.top + bounds.bottom)) / 2,
        size,
      };
      const end = heroRange + Math.max(0, viewport.offsetHeight - innerHeight);
      if (scroll <= end) {
        const t = ease((p - 0.06) / 0.7);
        const origin = {
          ...anchors[0],
          top: anchors[0].top + viewport.getBoundingClientRect().top,
        };
        state.frame = mix(origin, gallery, t);
        // The position describes a gentle spiral; orientation completes one turn.
        const arc = Math.sin(t * Math.PI);
        state.frame.left +=
          Math.sin(t * Math.PI * 2) * innerWidth * 0.035 * arc;
        state.frame.top +=
          Math.cos(t * Math.PI * 2) * innerHeight * 0.025 * arc;
        state.turn = state.motion ? ease((p - 0.1) / 0.75) * Math.PI * 2 : 0;
      } else {
        state.frame = mix(
          gallery,
          { ...anchors[1], top: anchors[1].top - scroll },
          ease((scroll - end) / (anchors[1].scroll - end)),
        );
        state.turn = state.motion ? Math.PI * 2 : 0;
      }
    } else {
      let index = 0;
      while (index < anchors.length - 2 && scroll > anchors[index + 1].scroll)
        index++;
      const a = anchors[index];
      const b = anchors[index + 1];
      // Hold subsequent compositions while reading; travel near section boundaries.
      const start =
        b.scroll - Math.min(innerHeight * 0.75, (b.scroll - a.scroll) * 0.55);
      state.frame = mix(a, b, ease((scroll - start) / (b.scroll - start)));
      state.frame.top -= scroll;
    }
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
