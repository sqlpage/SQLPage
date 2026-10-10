import { initSqlHighlight } from "./sql-highlight.js";

/** Prepare the next live example before changing the visible preview and SQL. */
export function initComponentDemos(root, tabs, signal) {
  const panel = root.querySelector("#demo-panel");
  const viewport = panel.querySelector(".demo-preview");
  const source = root.querySelector("#demo-source");
  const status = root.querySelector(".demo-status");
  const renderSource = initSqlHighlight(source, signal);
  const media = matchMedia("(prefers-reduced-motion: reduce)");
  const picker = root.querySelector(".component-picker");
  let preview = viewport.querySelector("iframe");
  let component = "table";
  let sourceFiles = { main: source.textContent };
  let request;
  let pending;
  let animation;
  let fade;
  let resizePreview;

  function resizePanel(change, entering = false) {
    const before = panel.getBoundingClientRect().height;
    animation?.cancel();
    fade?.cancel();
    change();
    const after = panel.getBoundingClientRect().height;
    if (media.matches) return;
    animation = panel.animate(
      [
        { height: `${before}px`, opacity: entering ? 0 : 1 },
        { height: `${after}px`, opacity: 1 },
      ],
      { duration: 320, easing: "cubic-bezier(0.22, 1, 0.36, 1)" },
    );
  }

  function fitPreview() {
    resizePreview?.disconnect();
    const content = preview.contentDocument?.querySelector(
      "#sqlpage_main_wrapper",
    );
    if (!content) return;
    const fit = () => {
      const height = `${Math.ceil(content.getBoundingClientRect().height) + 2}px`;
      if (viewport.style.height !== height)
        resizePanel(() => {
          viewport.style.height = height;
        });
    };
    resizePreview = new ResizeObserver(fit);
    resizePreview.observe(content);
    fit();
  }
  preview.addEventListener("load", fitPreview, { signal });
  fitPreview();

  root.querySelector("[data-copy-sql]").addEventListener(
    "click",
    async () => {
      try {
        await navigator.clipboard.writeText(source.textContent);
        status.textContent = "SQL copied.";
      } catch {
        status.textContent = "Copy unavailable. Select the SQL below.";
      }
    },
    { signal },
  );
  tabs(
    root.querySelector(".source-tabs"),
    "aria-selected",
    (button) => {
      source.setAttribute("aria-labelledby", button.id);
      status.textContent = "";
      resizePanel(() => {
        void renderSource(sourceFiles[button.dataset.file]);
      });
    },
    signal,
  );

  function loadPreview(frame, requestSignal) {
    return new Promise((resolve, reject) => {
      let observer;
      const abort = () => {
        observer?.disconnect();
        reject(new DOMException("Demo cancelled", "AbortError"));
      };
      requestSignal.addEventListener("abort", abort, { once: true });
      frame.addEventListener(
        "load",
        async () => {
          const document = frame.contentDocument;
          if (!document?.querySelector("#sqlpage_main_wrapper")) {
            reject(new Error("Preview unavailable"));
            return;
          }
          const ready = () => {
            if (document.querySelector("[data-pre-init]")) return;
            observer?.disconnect();
            requestSignal.removeEventListener("abort", abort);
            resolve();
          };
          await document.fonts.ready;
          if (requestSignal.aborted) return;
          observer = new MutationObserver(ready);
          observer.observe(document, { attributes: true, subtree: true });
          ready();
        },
        { once: true, signal: requestSignal },
      );
    });
  }

  async function selectDemo(button) {
    request?.abort();
    pending?.remove();
    fade?.cancel();
    const next = button.dataset.demo;
    if (next === component) {
      panel.removeAttribute("aria-busy");
      return;
    }
    const controller = new AbortController();
    request = controller;
    const incoming = preview.cloneNode(false);
    pending = incoming;
    incoming.title = "Loading SQLPage component demo";
    incoming.setAttribute("aria-hidden", "true");
    incoming.inert = true;
    incoming.loading = "eager";
    const loaded = loadPreview(incoming, controller.signal);
    incoming.src = `/landing-demos/demo.sql?component=${next}`;
    viewport.append(incoming);
    panel.setAttribute("aria-busy", "true");
    status.textContent = "";
    try {
      const [files] = await Promise.all([
        next === "catalog"
          ? null
          : fetch(`/landing-demos/source.sql?component=${next}`, {
              signal: controller.signal,
            }).then((response) => {
              if (!response.ok) throw new Error("Source unavailable");
              return response.json();
            }),
        loaded,
      ]);
      if (controller.signal.aborted) return;
      if (!media.matches) {
        fade = panel.animate([{ opacity: 1 }, { opacity: 0 }], {
          duration: 100,
          fill: "forwards",
        });
        await fade.finished;
      }
      if (controller.signal.aborted) return;
      resizePreview?.disconnect();
      preview.removeEventListener("load", fitPreview);
      const outgoing = preview;
      resizePanel(() => {
        outgoing.remove();
        preview = incoming;
        pending = null;
        component = next;
        incoming.title = "Live SQLPage component demo";
        incoming.removeAttribute("aria-hidden");
        incoming.inert = false;
        viewport.style.height = `${Math.ceil(incoming.contentDocument.querySelector("#sqlpage_main_wrapper").getBoundingClientRect().height) + 2}px`;
        panel.dataset.demo = next;
        panel.setAttribute("aria-labelledby", button.id);
        root.querySelector('[data-file="save"]').hidden = next !== "form";
        root.querySelector("#demo-filename").textContent = `${next}.sql`;
        const link = panel.querySelector(".demo-docs");
        link.href = `/component.sql?component=${next}`;
        link.firstChild.textContent = `Explore the ${next} component `;
        if (files) {
          sourceFiles = { main: files.source, save: files.save_source };
          // Avoid an intermediate one-line loading state or a nested resize.
          const main = root.querySelector("#demo-filename");
          for (const tab of root.querySelectorAll(".source-tabs button")) {
            tab.setAttribute("aria-selected", String(tab === main));
            tab.tabIndex = tab === main ? 0 : -1;
          }
          source.setAttribute("aria-labelledby", main.id);
          void renderSource(sourceFiles.main);
        }
      }, true);
      preview.addEventListener("load", fitPreview, { signal });
      fitPreview();
      panel.removeAttribute("aria-busy");
    } catch {
      if (controller.signal.aborted) return;
      controller.abort();
      incoming.remove();
      fade?.cancel();
      panel.removeAttribute("aria-busy");
      for (const tab of picker.querySelectorAll("button")) {
        const selected = tab.dataset.demo === component;
        tab.setAttribute("aria-selected", String(selected));
        tab.tabIndex = selected ? 0 : -1;
      }
      status.textContent = "The demo could not load. Please try again.";
    }
  }
  tabs(picker, "aria-selected", (button) => void selectDemo(button), signal);
  media.addEventListener(
    "change",
    () => {
      if (media.matches) animation?.cancel();
    },
    { signal },
  );
  return () => {
    request?.abort();
    pending?.remove();
    resizePreview?.disconnect();
    animation?.cancel();
    fade?.cancel();
  };
}
