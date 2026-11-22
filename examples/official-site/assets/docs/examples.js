if (navigator.clipboard?.writeText) {
  for (const source of document.querySelectorAll(".docs-example-source")) {
    const button = source.querySelector("button");
    const label = button.querySelector("span");
    const code = source.querySelector("code");
    let reset;
    button.hidden = false;
    button.addEventListener("click", async () => {
      clearTimeout(reset);
      try {
        await navigator.clipboard.writeText(code.textContent);
        label.textContent = "Copied";
      } catch {
        label.textContent = "Select the SQL to copy";
      }
      reset = setTimeout(() => {
        label.textContent = "Copy SQL";
      }, 2500);
    });
  }
}
