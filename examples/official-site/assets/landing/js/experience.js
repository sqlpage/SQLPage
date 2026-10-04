import { initScrollProgress } from "./scroll-progress.js";

/** CDN and WebGL failures are isolated from ordinary page navigation. */
export function initExperience(root) {
  const section = root.querySelector(".experience");
  const mount = root.querySelector(".scene-canvas");
  const hitArea = root.querySelector(".model-interaction");
  const anchor = root.querySelector(".anchor-letter");
  const error = root.querySelector(".scene-error");
  const media = matchMedia("(prefers-reduced-motion: reduce)");
  const state = { motion: !media.matches };
  const events = new AbortController();
  let controller, stopScroll;
  let generation = 0;
  let disposed = false;
  function syncMotion() {
    state.motion = !media.matches;
    root.dataset.motion = state.motion ? "on" : "off";
  }
  function setStatus(status) {
    root.dataset.scene = status;
    error.hidden = status !== "error";
    hitArea.dataset.ready = String(status === "ready");
    hitArea.inert = status !== "ready";
    hitArea.tabIndex = status === "ready" ? 0 : -1;
  }
  function stopScene() {
    controller?.dispose();
    controller = undefined;
  }
  async function startScene() {
    const attempt = ++generation;
    stopScene();
    setStatus("loading");
    try {
      const { createDatabaseScene } = await import("./scene/database-scene.js");
      if (disposed || attempt !== generation) return;
      controller = createDatabaseScene({
        mount,
        hitArea,
        anchor,
        getState: () => state,
        onReady() {
          if (disposed || attempt !== generation) return;
          setStatus("ready");
        },
        onError() {
          if (disposed || attempt !== generation) return;
          setStatus("error");
          // Allow a synchronous constructor callback to return before disposal.
          queueMicrotask(() => {
            if (!disposed && attempt === generation) stopScene();
          });
        },
      });
    } catch (cause) {
      if (disposed || attempt !== generation) return;
      console.error("SQLPage scene failed to load", cause);
      stopScene();
      setStatus("error");
    }
  }
  stopScroll = initScrollProgress(section, mount, state);
  syncMotion();
  media.addEventListener("change", syncMotion, { signal: events.signal });
  root
    .querySelector("[data-retry]")
    .addEventListener("click", startScene, { signal: events.signal });
  root
    .querySelector(".brand")
    .addEventListener("click", () => controller?.reset(), {
      signal: events.signal,
    });
  void startScene();
  return () => {
    disposed = true;
    generation++;
    events.abort();
    stopScroll?.();
    stopScene();
  };
}
