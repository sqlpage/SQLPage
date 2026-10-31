/** Deform shared SVG section edges with travelling swells and damped scroll impulses. */
const TAU = Math.PI * 2;
const WIDTH = 1200;
const STEP = 100;
// Speeds are radians/second; damping times are seconds; scroll uses viewport heights.
const IDLE_SPEED = 0.18;
const DRAG_TIME = 1.25;
const SETTLE_TIME = 2.4;
const WIND_GAIN = 4;
const MAX_WIND = 3;
const DISTURBANCE_GAIN = 5;

// Two travelling waves interfere, so the surface changes shape as well as
// travelling sideways. Cubic tangents keep the small set of samples smooth.
function surfacePath(phase, swell, back) {
  const amplitude = (back ? 25 : 30) + swell * 6;
  const frequency = TAU / (back ? 920 : 1050);
  const rippleFrequency = TAU / 430;
  const ripplePhase = -phase * 1.4;
  const baseline = back ? 43 : 50;
  const ripple = (back ? 3 : 4) + swell * 6;
  const samples = [];
  for (let x = 0; x <= WIDTH; x += STEP) {
    const angle = x * frequency + phase;
    const rippleAngle = x * rippleFrequency + ripplePhase;
    samples.push({
      x,
      y:
        baseline + amplitude * Math.sin(angle) + ripple * Math.sin(rippleAngle),
      slope:
        amplitude * frequency * Math.cos(angle) +
        ripple * rippleFrequency * Math.cos(rippleAngle),
    });
  }
  const number = (value) => value.toFixed(2);
  let path = `M0 ${number(samples[0].y)}`;
  for (let i = 1; i < samples.length; i++) {
    const previous = samples[i - 1];
    const current = samples[i];
    const handle = STEP / 3;
    path += `C${number(previous.x + handle)} ${number(previous.y + previous.slope * handle)} ${number(current.x - handle)} ${number(current.y - current.slope * handle)} ${current.x} ${number(current.y)}`;
  }
  // Both layers always extend below the SVG: no gaps can open at the seam.
  return `${path}V110H0Z`;
}

export function initWaveMotion(root) {
  const waves = [...root.querySelectorAll(".section-wave")].map(
    (element, index) => ({
      element,
      paths: [...element.querySelectorAll("path")],
      phase: index * 0.9,
    }),
  );
  const visible = new Set();
  const motion = matchMedia("(prefers-reduced-motion: reduce)");
  const events = new AbortController();
  let frame = 0;
  let phase = 0;
  let wind = 0;
  let disturbance = 0;
  let previousTime = null;
  let previousScroll = scrollY;

  function advance(time) {
    const dt = previousTime === null ? 0 : (time - previousTime) / 1000;
    previousTime = time;
    const drag = Math.exp(-dt / DRAG_TIME);
    // Integrate exponential drag exactly, independent of animation frame rate.
    phase += IDLE_SPEED * dt + wind * DRAG_TIME * (1 - drag);
    wind *= drag;
    disturbance *= Math.exp(-dt / SETTLE_TIME);
  }

  function draw(wave) {
    const current = wave.phase + phase;
    for (const path of wave.paths) {
      const back = path.classList.contains("wave-back");
      path.setAttribute(
        "d",
        surfacePath(back ? -current * 0.8 + 1.2 : current, disturbance, back),
      );
    }
  }

  function schedule() {
    const paused = motion.matches || document.hidden;
    if (paused) previousTime = null;
    if (paused || !visible.size) {
      cancelAnimationFrame(frame);
      frame = 0;
    } else if (!frame) frame = requestAnimationFrame(render);
  }

  function render() {
    frame = 0;
    advance(performance.now());
    for (const wave of visible) draw(wave);
    schedule();
  }

  function onScroll() {
    const distance = (scrollY - previousScroll) / Math.max(1, innerHeight);
    previousScroll = scrollY;
    if (motion.matches || document.hidden) return;
    // Offscreen water still receives wind; elapsed drag is applied lazily here
    // or on its next visible frame, without running an offscreen render loop.
    advance(performance.now());
    // Scroll pushes the water, rather than choosing its pose. Faster scrolling
    // adds impulses before they dissipate; reversing never resets the phase.
    wind = Math.max(-MAX_WIND, Math.min(MAX_WIND, wind + distance * WIND_GAIN));
    disturbance =
      1 - (1 - disturbance) * Math.exp(-Math.abs(distance) * DISTURBANCE_GAIN);
    schedule();
  }

  const observer = new IntersectionObserver(
    (entries) => {
      for (const { target, isIntersecting } of entries) {
        const wave = waves.find(({ element }) => element === target);
        target.toggleAttribute("data-wave-visible", isIntersecting);
        if (isIntersecting) visible.add(wave);
        else visible.delete(wave);
      }
      schedule();
    },
    // Ignore the two-pixel seam once a section has completely covered the page.
    { rootMargin: "-3px 0px 0px" },
  );
  for (const wave of waves) {
    draw(wave);
    observer.observe(wave.element);
  }
  const { signal } = events;
  window.addEventListener("scroll", onScroll, { passive: true, signal });
  document.addEventListener("visibilitychange", schedule, { signal });
  motion.addEventListener("change", schedule, { signal });

  return () => {
    cancelAnimationFrame(frame);
    observer.disconnect();
    events.abort();
    for (const { element } of waves)
      element.removeAttribute("data-wave-visible");
  };
}
