// Code remains readable while the optional highlighter loads.
if (document.querySelector("pre code")) {
  highlightCode().catch(() => {
    // The CDN may be unavailable.
  });
}

async function highlightCode() {
  const base =
    "https://cdn.jsdelivr.net/npm/@highlightjs/cdn-assets@11.11.1/es";
  const { default: engine } = await import(`${base}/highlight.min.js`);
  await Promise.all(
    ["sql", "json", "handlebars", "nginx"].map(async (language) => {
      if (engine.getLanguage(language)) return;
      if (!document.querySelector(`code.language-${language}`)) return;
      const { default: grammar } = await import(
        `${base}/languages/${language}.min.js`
      );
      engine.registerLanguage(language, grammar);
    }),
  );
  engine.highlightAll();
}
