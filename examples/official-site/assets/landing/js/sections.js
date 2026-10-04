import { initSqlHighlight } from "./sql-highlight.js";

/** Accessible tab groups share selection and keyboard behavior. */
function tabs(group, attribute, select, signal) {
  const buttons = [...group.querySelectorAll("button")];
  function activate(button) {
    for (const item of buttons) {
      item.setAttribute(attribute, String(item === button));
      item.tabIndex = item === button ? 0 : -1;
    }
    select(button);
  }
  group.addEventListener(
    "click",
    (event) => {
      const button = event.target.closest("button");
      if (buttons.includes(button)) activate(button);
    },
    { signal },
  );
  group.addEventListener(
    "keydown",
    (event) => {
      const visible = buttons.filter((button) => !button.hidden);
      const index = visible.indexOf(document.activeElement);
      if (index < 0) return;
      let next;
      if (["ArrowRight", "ArrowDown"].includes(event.key))
        next = (index + 1) % visible.length;
      if (["ArrowLeft", "ArrowUp"].includes(event.key))
        next = (index + visible.length - 1) % visible.length;
      if (event.key === "Home") next = 0;
      if (event.key === "End") next = visible.length - 1;
      if (next !== undefined) {
        event.preventDefault();
        activate(visible[next]);
        visible[next].focus();
      }
    },
    { signal },
  );
  return activate;
}

export function initSections(root) {
  const events = new AbortController();
  const { signal } = events;
  const panel = root.querySelector("#demo-panel");
  const source = root.querySelector("#demo-source");
  const status = root.querySelector(".demo-status");
  const renderSource = initSqlHighlight(source, signal);
  const preview = panel.querySelector("iframe");
  let resizePreview;
  function fitPreview() {
    resizePreview?.disconnect();
    const content = preview.contentDocument?.querySelector(
      "#sqlpage_main_wrapper",
    );
    if (!content) return;
    const fit = () => {
      preview.style.height = `${Math.ceil(content.getBoundingClientRect().height) + 2}px`;
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
  let sourceFiles = { main: source.textContent };
  const chooseSource = tabs(
    root.querySelector(".source-tabs"),
    "aria-selected",
    (button) => {
      source.setAttribute("aria-labelledby", button.id);
      status.textContent = "";
      void renderSource(
        sourceFiles[button.dataset.file] || "-- Loading source…",
      );
    },
    signal,
  );
  let request;
  async function loadSource(component) {
    request?.abort();
    request = new AbortController();
    status.textContent = "";
    try {
      const response = await fetch(
        `/landing-demos/source.sql?component=${component}`,
        { signal: request.signal },
      );
      if (!response.ok) throw new Error("Source unavailable");
      const data = await response.json();
      sourceFiles = { main: data.source, save: data.save_source };
      chooseSource(root.querySelector('.source-tabs [aria-selected="true"]'));
    } catch (error) {
      if (error.name !== "AbortError") {
        source.textContent =
          "-- Source unavailable. Open the component documentation below.";
        status.textContent = "The demo source could not load.";
      }
    }
  }
  tabs(
    root.querySelector(".component-picker"),
    "aria-selected",
    (button) => {
      const component = button.dataset.demo;
      panel.dataset.demo = component;
      root.querySelector('[data-file="save"]').hidden = component !== "form";
      root.querySelector("#demo-filename").textContent = `${component}.sql`;
      panel.setAttribute("aria-labelledby", button.id);
      panel.querySelector("iframe").src =
        `/landing-demos/demo.sql?component=${component}`;
      const link = panel.querySelector(".demo-docs");
      link.href = `/component.sql?component=${component}`;
      link.firstChild.textContent = `Explore the ${component} component `;
      if (component !== "catalog") {
        sourceFiles = {};
        chooseSource(root.querySelector("#demo-filename"));
        void loadSource(component);
      } else request?.abort();
    },
    signal,
  );
  tabs(
    root.querySelector('[aria-label="Application stack"]'),
    "aria-pressed",
    (button) => {
      const comparison = root.querySelector(".stack-comparison");
      comparison.dataset.stack = button.dataset.stack;
      for (const copy of comparison.querySelectorAll("[data-stack-copy]"))
        copy.setAttribute(
          "aria-hidden",
          String(copy.dataset.stackCopy !== button.dataset.stack),
        );
      root.querySelector("[data-stack-caption]").textContent =
        button.dataset.stack === "sqlpage"
          ? "-- Same five layers. SQLPage handles the plumbing."
          : "-- 5 layers, all yours.";
    },
    signal,
  );
  tabs(
    root.querySelector('[aria-label="Deployment options"]'),
    "aria-selected",
    (button) => {
      for (const id of ["server", "hosting"])
        root.querySelector(`#${id}-panel`).hidden =
          id !== button.dataset.deployment;
    },
    signal,
  );
  return () => {
    events.abort();
    request?.abort();
    resizePreview?.disconnect();
  };
}
