import { copyCode } from "../copy-code.js";

function enableCopy(button, code, kind) {
  const label = button.querySelector("[aria-live]") ?? button;
  const initialLabel = label.textContent;
  let reset;
  button.hidden = false;
  button.addEventListener("click", async () => {
    clearTimeout(reset);
    label.textContent = await copyCode(code, kind);
    reset = setTimeout(() => {
      label.textContent = initialLabel;
    }, 2500);
  });
}

for (const source of document.querySelectorAll(".docs-example-source")) {
  enableCopy(
    source.querySelector("button"),
    source.querySelector("code"),
    "SQL",
  );
}

// Attach tools only to standalone snippets, leaving live previews and their source intact.
const toolbar = document.getElementById("docs-snippet-tools");
for (const code of document.querySelectorAll(
  ".docs-main pre > code:not(.docs-example-source *, .docs-example-output *)",
)) {
  const pre = code.parentElement;
  const tools = toolbar.content.cloneNode(true);
  const language = Array.from(code.classList).find((name) =>
    name.startsWith("language-"),
  );
  if (language)
    tools.querySelector("span").textContent = language.slice(
      "language-".length,
    );
  enableCopy(tools.querySelector("button"), code, "Code");
  pre.before(tools);
  pre.tabIndex = 0;
}
