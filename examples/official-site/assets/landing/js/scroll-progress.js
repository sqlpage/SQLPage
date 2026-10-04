/** Batch scroll measurements into one frame; never intercept wheel events. */
export function initScrollProgress(section, mount, state) {
  const events = new AbortController();
  const { signal } = events;
  const intro = section.querySelector(".intro-row");
  const ribbon = section.querySelector(".sql-ribbon");
  let frame = 0;
  function update() {
    frame = 0;
    const rect = section.getBoundingClientRect();
    const viewportHeight = mount.clientHeight;
    const entryOverflow = Math.max(0, viewportHeight - window.innerHeight);
    const progress = Math.min(
      1,
      Math.max(
        0,
        (-rect.top - entryOverflow) / Math.max(1, rect.height - viewportHeight),
      ),
    );
    state.progress = progress;
    section.style.setProperty("--progress", String(progress));
    section.style.setProperty(
      "--intro-opacity",
      String(1 - Math.min(1, Math.max(0, (progress - 0.06) / 0.35))),
    );
    section.style.setProperty("--intro-shift", `${-progress * 90}px`);
    section.style.setProperty(
      "--ribbon-opacity",
      String(1 - Math.min(1, progress / 0.38)),
    );
    intro.inert = progress > 0.4;
    ribbon.inert = progress > 0.37;
  }
  const schedule = () => {
    if (!frame) frame = requestAnimationFrame(update);
  };
  window.addEventListener("scroll", schedule, { passive: true, signal });
  window.addEventListener("resize", schedule, { signal });
  window.addEventListener("sqlpage:layout", schedule, { signal });
  update();
  return () => {
    events.abort();
    cancelAnimationFrame(frame);
    state.progress = 0;
    for (const property of [
      "--progress",
      "--intro-opacity",
      "--intro-shift",
      "--ribbon-opacity",
    ])
      section.style.removeProperty(property);
    intro.inert = false;
    ribbon.inert = false;
  };
}
