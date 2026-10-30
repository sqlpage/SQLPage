/** Synchronize the illustrative streaming timelines and suspend invisible animation. */
// Both timelines share an illustrative clock; these are not benchmark timings.
const DURATION = 7000;

export function initInstantMotion(root) {
  const diagram = root.querySelector(".instant-diagram");
  const button = diagram?.querySelector(".timeline-toggle");
  if (!button) return () => {};
  const motion = matchMedia("(prefers-reduced-motion: reduce)");
  const events = new AbortController();
  const { signal } = events;
  let visible = false;
  let paused = false;
  let animations = [];

  function animate(element, keyframes) {
    animations.push(
      element.animate(keyframes, {
        duration: DURATION,
        iterations: Infinity,
      }),
    );
  }

  function fill(element, start, end) {
    animate(element, [
      { clipPath: "inset(0 100% 0 0)", offset: 0 },
      { clipPath: "inset(0 100% 0 0)", offset: start },
      { clipPath: "inset(0 0% 0 0)", offset: end },
      { clipPath: "inset(0 0% 0 0)", opacity: 1, offset: 0.88 },
      { clipPath: "inset(0 0% 0 0)", opacity: 0, offset: 0.96 },
      { clipPath: "inset(0 100% 0 0)", opacity: 0, offset: 1 },
    ]);
  }

  function reset() {
    for (const animation of animations) animation.cancel();
    animations = [];
    delete diagram.dataset.streamMotion;
  }

  function update() {
    button.hidden = motion.matches;
    button.setAttribute(
      "aria-label",
      paused ? "Play streaming comparison" : "Pause streaming comparison",
    );
    button.dataset.paused = String(paused);
    if (motion.matches || !visible) {
      reset();
      return;
    }
    if (!animations.length) {
      diagram.dataset.streamMotion = "true";
      const stages = diagram.querySelectorAll(
        ".render-timeline .timeline-fill",
      );
      const stops = [0.025, 0.14, 0.36, 0.66, 0.78];
      stages.forEach((stage, index) => {
        fill(stage, stops[index], stops[index + 1]);
      });
      fill(diagram.querySelector(".stream-fill"), stops[0], stops[1]);
      animate(diagram.querySelector(".stream-sheen"), [
        { transform: "translateX(-300%)", opacity: 0, offset: 0 },
        { transform: "translateX(-300%)", opacity: 0, offset: 0.14 },
        { transform: "translateX(-100%)", opacity: 0.6, offset: 0.2 },
        { transform: "translateX(400%)", opacity: 0.6, offset: 0.72 },
        { transform: "translateX(400%)", opacity: 0, offset: 0.78 },
        { transform: "translateX(400%)", opacity: 0, offset: 1 },
      ]);
      diagram
        .querySelectorAll(".render-labels span")
        .forEach((label, index) => {
          const start = 0.12 + index * 0.04;
          animate(label, [
            { opacity: 0.35, offset: 0 },
            { opacity: 0.35, offset: start },
            { opacity: 1, offset: start + 0.04 },
            { opacity: 1, offset: 0.88 },
            { opacity: 0.35, offset: 0.96 },
            { opacity: 0.35, offset: 1 },
          ]);
        });
      // Synchronize all fills and labels to the same document timeline.
      const startTime = document.timeline.currentTime;
      for (const animation of animations) animation.startTime = startTime;
    }
    for (const animation of animations) {
      if (paused || document.hidden) animation.pause();
      else animation.play();
    }
  }

  const observer = new IntersectionObserver(
    ([entry]) => {
      visible = entry.isIntersecting;
      update();
    },
    { threshold: 0.25 },
  );
  observer.observe(diagram);
  button.addEventListener(
    "click",
    () => {
      paused = !paused;
      update();
    },
    { signal },
  );
  motion.addEventListener("change", update, { signal });
  document.addEventListener("visibilitychange", update, { signal });
  return () => {
    events.abort();
    observer.disconnect();
    reset();
    button.hidden = true;
  };
}
