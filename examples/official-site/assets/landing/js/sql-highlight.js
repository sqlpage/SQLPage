/** Use the site's existing Highlight.js engine with only its SQL grammar. */
export function initSqlHighlight(element, signal) {
  let engine;
  const ready = Promise.all([
    import(
      "https://cdn.jsdelivr.net/npm/@highlightjs/cdn-assets@11.11.1/es/core.min.js"
    ),
    import(
      "https://cdn.jsdelivr.net/npm/@highlightjs/cdn-assets@11.11.1/es/languages/sql.min.js"
    ),
  ])
    .then(([core, sql]) => {
      engine = core.default;
      engine.registerLanguage("sql", sql.default);
      paint();
    })
    .catch(() => {
      /* Plain SQL stays readable if the optional CDN is unavailable. */
    });
  function paint() {
    if (signal.aborted) return;
    const lines = element.textContent.trimEnd().split("\n");
    const fragment = document.createDocumentFragment();
    for (const [index, line] of lines.entries()) {
      const row = document.createElement("span");
      row.className = "sql-line";
      if (engine)
        row.innerHTML = engine.highlight(line, { language: "sql" }).value;
      else row.textContent = line;
      fragment.append(row);
      if (index < lines.length - 1) fragment.append("\n");
    }
    element.replaceChildren(fragment);
  }
  paint();
  return (code) => {
    element.textContent = code;
    paint();
    return ready;
  };
}
