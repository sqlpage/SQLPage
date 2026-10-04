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
    if (engine && !signal.aborted)
      element.innerHTML = engine.highlight(element.textContent, {
        language: "sql",
      }).value;
  }
  return (code) => {
    element.textContent = code;
    paint();
    return ready;
  };
}
