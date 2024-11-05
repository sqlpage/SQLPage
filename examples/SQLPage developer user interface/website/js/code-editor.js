const cdn = "https://cdn.jsdelivr.net/npm/monaco-editor@0.50.0/";

function on_monaco_load() {
  // Create an editor div, display it after the '#code-editor' textarea, hide the textarea, and create a Monaco editor in the div with the contents of the textarea
  // When the form is submitted, set the value of the textarea to the value of the Monaco editor
  const textarea = document.getElementById("code-editor");
  const editorDiv = document.createElement("div");
  editorDiv.style.width = "100%";
  editorDiv.style.height = "700px";
  textarea.parentNode.insertBefore(editorDiv, textarea.nextSibling);
  const monacoConfig = {
    value: textarea.value,
    language: "sql",
  };

  self.MonacoEnvironment = {
    baseUrl: `${cdn}min/`,
  };
  const editor = monaco.editor.create(editorDiv, monacoConfig);
  textarea.style.display = "none";
  const form = textarea.form;
  form.addEventListener("submit", () => {
    textarea.value = editor.getValue();
  });
}

function set_require_config() {
  require.config({ paths: { vs: `${cdn}min/vs` } });
  require(["vs/editor/editor.main"], on_monaco_load);
}
const loader_script = document.createElement("script");
loader_script.src = `${cdn}min/vs/loader.js`;
loader_script.onload = set_require_config;
document.head.appendChild(loader_script);
