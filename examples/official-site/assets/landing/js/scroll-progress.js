/** Measure authored sculpture anchors, then update flight, fading handoffs and hero text. */
import { layoutLandingFrame, SCULPTURE_FRAME } from "./scene/landing-frame.js";

/** One sculpture: an opening flight, then page anchors with fading handoffs. */
export function initScrollProgress(section, mount, state) {
  const root = section.closest(".sqlpage-world");
  const preview = root.querySelector(".scene-preview");
  const layer = root.querySelector(".scene-layer");
  const stops = [...root.querySelectorAll("[data-scene-stop]")];
  const events = new AbortController();
  let frame = 0;
  let heroFrame;
  let anchors = [];
  let heroRange = 0;
  let heroDockScroll = 0;
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
    // Read authored page positions even when earlier sections are held under a cover.
    root.setAttribute("data-measuring-layout", "");
    // Measure the authored hero pose, independent of its current text fade/shift.
    section.style.setProperty("--progress", "0");
    section.style.setProperty("--intro-shift", "0px");
    heroFrame = layoutLandingFrame(mount, root.querySelector(".anchor-letter"));
    anchors = [];
    heroRange = Math.max(0, section.offsetHeight - viewport.offsetHeight);
    const maxScroll = Math.max(
      0,
      document.documentElement.scrollHeight - innerHeight,
    );
    for (const stop of stops) {
      const rect = stop.getBoundingClientRect();
      const sheet = stop.closest(".landing-section");
      const sheetRect = sheet.getBoundingClientRect();
      const nextSheet = sheet.nextElementSibling;
      // Measure the visible model, not its much larger square canvas.
      const size = rect.width / SCULPTURE_FRAME.bodyWidth;
      const bodyHeight =
        size * (SCULPTURE_FRAME.bounds.bottom - SCULPTURE_FRAME.bounds.top);
      const bodyTop = rect.top + scrollY;
      const fadeDistance = Math.min(innerHeight * 0.12, bodyHeight * 0.45);
      const heldTop = root.hasAttribute("data-section-motion")
        ? rect.top -
          sheetRect.top +
          (parseFloat(sheet.style.getPropertyValue("--section-stick-top")) || 0)
        : -Infinity;
      // A held section can leave part of its sculpture on screen. In that case
      // its lifetime ends when the next wave covers it, not at its authored Y.
      let exitScroll =
        heldTop + bodyHeight > 0 ? Infinity : bodyTop + bodyHeight;
      if (nextSheet?.matches(".landing-section")) {
        const waveHeight = parseFloat(
          getComputedStyle(nextSheet).getPropertyValue("--wave-height"),
        );
        const coverScroll =
          nextSheet.getBoundingClientRect().top +
          scrollY -
          waveHeight * 0.75 -
          (heldTop + bodyHeight * 0.5);
        exitScroll = Math.min(exitScroll, coverScroll);
      }
      anchors.push({
        element: stop,
        enterScroll: Math.min(bodyTop - innerHeight, maxScroll - fadeDistance),
        exitScroll,
        fadeDistance,
        bodyHeight,
        bodyTop,
        left: rect.left - size * SCULPTURE_FRAME.bounds.left,
        top: bodyTop - size * SCULPTURE_FRAME.bounds.top,
        size,
      });
    }
    heroDockScroll = Math.min(
      maxScroll,
      anchors[0].bodyTop - innerHeight * 0.25,
    );
    anchors[0].enterScroll = -Infinity; // The opening flight supplies its entrance.
    anchors.at(-1).exitScroll = Infinity; // Keep the closing composition visible.
    for (let index = 1; index < anchors.length; index++) {
      const previous = anchors[index - 1];
      const next = anchors[index];
      // When both anchors fit on screen, hand over as the new sculpture gains
      // room at the bottom edge. Otherwise keep the old one until it leaves.
      // The two short fades meet at zero, hiding the change of page position.
      previous.exitScroll = Math.min(
        previous.exitScroll,
        next.enterScroll + next.fadeDistance,
        maxScroll - next.fadeDistance,
      );
      next.enterScroll = Math.max(next.enterScroll, previous.exitScroll);
    }
    root.removeAttribute("data-measuring-layout");
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
    state.turn = state.motion ? (scroll / innerHeight) * Math.PI * 2 : 0;
    let sceneOpacity = 1;
    if (!heroRange && scroll < heroDockScroll) {
      // Reduced motion keeps the opening sculpture attached to the hero layout.
      state.frame = {
        ...heroFrame,
        top: heroFrame.top + viewport.getBoundingClientRect().top,
      };
    } else if (heroRange && scroll <= heroDockScroll) {
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
          ...heroFrame,
          top: heroFrame.top + viewport.getBoundingClientRect().top,
        };
        state.frame = mix(origin, gallery, t);
        // The position describes a gentle spiral while rotation follows scroll.
        const arc = Math.sin(t * Math.PI);
        state.frame.left +=
          Math.sin(t * Math.PI * 2) * innerWidth * 0.035 * arc;
        state.frame.top +=
          Math.cos(t * Math.PI * 2) * innerHeight * 0.025 * arc;
      } else {
        const t = ease((scroll - end) / (heroDockScroll - end));
        state.frame = mix(
          { ...gallery, top: gallery.top + end },
          anchors[0],
          t,
        );
        state.frame.left += Math.sin(t * Math.PI) * innerWidth * 0.025;
        state.frame.top += Math.sin(t * Math.PI) ** 2 * innerHeight * 0.06;
        state.frame.top -= scroll;
      }
    } else {
      let index = 0;
      while (
        index < anchors.length - 1 &&
        scroll >= anchors[index + 1].enterScroll
      )
        index++;
      const anchor = anchors[index];
      // Follow the actual page anchor, including its section holding beneath the
      // next cover. Only the opening flight interpolates position or size.
      const rect = anchor.element.getBoundingClientRect();
      if (state.motion) {
        sceneOpacity = Math.min(
          ease((scroll - anchor.enterScroll) / anchor.fadeDistance),
          ease((anchor.exitScroll - scroll) / anchor.fadeDistance),
          // Fade only the last part leaving the top, rather than fading the
          // entire model as soon as its jewel approaches the viewport edge.
          ease((rect.top + anchor.bodyHeight) / anchor.fadeDistance),
          ease((innerHeight - rect.top) / anchor.fadeDistance),
        );
      }
      state.frame = {
        left: rect.left - anchor.size * SCULPTURE_FRAME.bounds.left,
        top: rect.top - anchor.size * SCULPTURE_FRAME.bounds.top,
        size: anchor.size,
      };
    }
    state.opacity = sceneOpacity;
    layer.style.opacity = String(sceneOpacity);
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
