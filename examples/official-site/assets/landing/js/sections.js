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
      const index = buttons.indexOf(document.activeElement);
      if (index < 0) return;
      let next;
      if (["ArrowRight", "ArrowDown"].includes(event.key))
        next = (index + 1) % buttons.length;
      if (["ArrowLeft", "ArrowUp"].includes(event.key))
        next = (index + buttons.length - 1) % buttons.length;
      if (event.key === "Home") next = 0;
      if (event.key === "End") next = buttons.length - 1;
      if (next !== undefined) {
        event.preventDefault();
        activate(buttons[next]);
        buttons[next].focus();
      }
    },
    { signal },
  );
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
      void renderSource(data.source);
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
      panel.setAttribute("aria-labelledby", button.id);
      panel.querySelector("iframe").src =
        `/landing-demos/demo.sql?component=${component}`;
      const link = panel.querySelector(".demo-docs");
      link.href = `/component.sql?component=${component}`;
      link.firstChild.textContent = `Explore the ${component} component `;
      void loadSource(component);
    },
    signal,
  );
  tabs(
    root.querySelector('[aria-label="Application stack"]'),
    "aria-pressed",
    (button) => {
      root.querySelector(".stack-comparison").dataset.stack =
        button.dataset.stack;
      root.querySelector("[data-stack-caption]").textContent =
        button.dataset.stack === "sqlpage"
          ? "-- Your SQL files and your database. That's the stack."
          : "-- 5 layers, all yours. Illustrative, not a benchmark.";
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
