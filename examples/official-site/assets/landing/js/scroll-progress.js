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
  function travel(a, b, t, distance) {
    // Spend most of the reading interval near the stops, with a quick passage
    // between them. A small drift keeps the sculpture alive at either landing.
    // The first 70% belongs to the current section, the last 30% is a
    // transition. Unlike easing the whole interval, this creates a real dwell.
    const progress = ease((t - 0.7) / 0.3);
    const pose = mix(a, b, progress);
    const arc = Math.sin(Math.PI * progress) ** 2;
    const bounds = SCULPTURE_FRAME.bounds;
    const center = bounds.left + SCULPTURE_FRAME.bodyWidth / 2;
    // Travel through the outer gutter, compact enough to clear demos and copy.
    // The envelope also slows the gutter detour and scale change at each stop.
    const size = pose.size;
    pose.size +=
      (Math.min(size, (innerWidth * 0.12) / SCULPTURE_FRAME.bodyWidth) - size) *
      arc;
    pose.left += (size - pose.size) * center;
    pose.top += ((size - pose.size) * (bounds.top + bounds.bottom)) / 2;
    pose.left += (innerWidth * 0.92 - (pose.left + pose.size * center)) * arc;
    const centerY = pose.top + (pose.size * (bounds.top + bounds.bottom)) / 2;
    const corridorY = Math.max(
      innerHeight * 0.3,
      Math.min(innerHeight * 0.7, centerY),
    );
    pose.top += (corridorY - centerY - innerHeight * 0.08) * arc;
    // Only a few pixels of vertical drift while reading; rotation is independent.
    pose.top -= Math.sin(t * Math.PI * 2) * Math.min(8, distance * 0.004);
    return pose;
  }
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
    // Unwrapped rotation is continuous across every section and reverses naturally.
    state.turn = state.motion ? (scroll / innerHeight) * Math.PI * 0.8 : 0;
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
        const t = ease(p);
        const origin = {
          ...anchors[0],
          top: anchors[0].top + viewport.getBoundingClientRect().top,
        };
        state.frame = mix(origin, gallery, t);
        // The position describes a gentle spiral while rotation follows scroll.
        const arc = Math.sin(t * Math.PI);
        state.frame.left +=
          Math.sin(t * Math.PI * 2) * innerWidth * 0.035 * arc;
        state.frame.top +=
          Math.cos(t * Math.PI * 2) * innerHeight * 0.025 * arc;
      } else {
        state.frame = travel(
          gallery,
          { ...anchors[1], top: anchors[1].top - anchors[1].scroll },
          clamp((scroll - end) / (anchors[1].scroll - end)),
          anchors[1].scroll - end,
        );
      }
    } else {
      let index = 0;
      while (index < anchors.length - 2 && scroll > anchors[index + 1].scroll)
        index++;
      const a = anchors[index];
      const b = anchors[index + 1];
      const t = clamp((scroll - a.scroll) / (b.scroll - a.scroll));
      state.frame = state.motion
        ? travel(
            { ...a, top: a.top - a.scroll },
            { ...b, top: b.top - b.scroll },
            t,
            b.scroll - a.scroll,
          )
        : mix(a, b, ease(t));
      if (!state.motion) state.frame.top -= scroll;
      if (state.motion && scroll > b.scroll) {
        // Keep the last composition gently orbiting while scrolling toward the footer.
        const tail = (scroll - b.scroll) / innerHeight;
        state.frame.left += Math.sin(tail * 2) * innerWidth * 0.025;
        state.frame.top -= Math.sin(tail * 2) * innerHeight * 0.02;
      }
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
