/** Select the code for the browser's Copy command if clipboard access is unavailable. */
export async function copyCode(code, label) {
  try {
    await navigator.clipboard.writeText(code.textContent);
    return `${label} copied.`;
  } catch {
    code.closest("[tabindex]")?.focus({ preventScroll: true });
    const range = document.createRange();
    range.selectNodeContents(code);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    return `${label} selected. Use Copy.`;
  }
}
