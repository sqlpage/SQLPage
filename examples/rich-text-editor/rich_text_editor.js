import Quill from "https://esm.sh/quill@2.0.3";
import { toMarkdown as mdastUtilToMarkdown } from "https://esm.sh/mdast-util-to-markdown@2.1.2";

/**
 * Converts Quill Delta object to a Markdown string using mdast.
 * @param {object} delta - Quill Delta object (https://quilljs.com/docs/delta/).
 * @returns {string} - Markdown representation.
 */
function deltaToMarkdown(delta) {
  try {
    const mdastTree = deltaToMdast(delta);
    const options = {
      bullet: "*",
      listItemIndent: "one",
      handlers: {},
      unknownHandler: (node) => {
        console.warn(`Unknown node type encountered: ${node.type}`, node);
        return false;
      },
    };
    return mdastUtilToMarkdown(mdastTree, options);
  } catch (error) {
    console.error("Error during Delta to Markdown conversion:", error);
    console.warn("Falling back to basic text extraction");
    return extractPlainTextFromDelta(delta);
  }
}

function extractPlainTextFromDelta(delta) {
  try {
    return delta.ops
      .map((op) => (typeof op.insert === "string" ? op.insert : ""))
      .join("")
      .trim();
  } catch (e) {
    console.error("Fallback extraction also failed:", e);
    return "";
  }
}

/**
 * Creates a div to replace the textarea and prepares it for Quill.
 * @param {HTMLTextAreaElement} textarea - The original textarea.
 * @returns {HTMLDivElement} - The div element created for the Quill editor.
 */
function createAndReplaceTextarea(textarea) {
  const editorDiv = document.createElement("div");
  editorDiv.className = "mb-3";
  editorDiv.style.height = "250px";

  const label = textarea.closest("label");
  if (!label) {
    textarea.parentNode.insertBefore(editorDiv, textarea);
  } else {
    label.parentNode.insertBefore(editorDiv, label.nextSibling);
  }
  textarea.style.display = "none";
  return editorDiv;
}

/**
 * Returns the toolbar options array configured for Markdown compatibility.
 * @returns {Array} - Quill toolbar options.
 */
function getMarkdownToolbarOptions() {
  return [
    [{ header: 1 }, { header: 2 }, { header: 3 }],
    ["bold", "italic"],
    ["link", "image", "blockquote", "code-block"],
    [{ list: "ordered" }, { list: "bullet" }],
    ["clean"],
  ];
}

/**
 * Initializes a Quill editor instance on a given div.
 * @param {HTMLDivElement} editorDiv - The div element for the editor.
 * @param {Array} toolbarOptions - The toolbar configuration.
 * @param {string} initialValue - The initial content for the editor.
 * @returns {Quill} - The initialized Quill instance.
 */
function initializeQuillEditor(editorDiv, toolbarOptions, initialValue) {
  const quill = new Quill(editorDiv, {
    theme: "snow",
    modules: {
      toolbar: toolbarOptions,
    },
  });
  if (initialValue) {
    quill.setText(initialValue);
  }
  return quill;
}

/**
 * Attaches a submit event listener to the form to update the hidden textarea.
 * @param {HTMLFormElement} form - The form containing the editor.
 * @param {HTMLTextAreaElement} textarea - The original (hidden) textarea.
 * @param {Quill} quill - The Quill editor instance.
 */
function updateTextareaOnSubmit(form, textarea, quill) {
  if (!form) {
    console.warn(
      "Textarea not inside a form, submission handling skipped for:",
      textarea.name || textarea.id,
    );
    return;
  }
  form.addEventListener("submit", () => {
    const delta = quill.getContents();
    const markdownContent = deltaToMarkdown(delta);
    textarea.value = markdownContent;
  });
}

function loadQuillStylesheet() {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = "https://esm.sh/quill@2.0.3/dist/quill.snow.css";
  document.head.appendChild(link);
}

function handleEditorInitError(textarea, error) {
  console.error("Failed to initialize Quill for textarea:", textarea, error);
  textarea.style.display = "";
  const errorMsg = document.createElement("p");
  errorMsg.textContent = "Failed to load rich text editor.";
  errorMsg.style.color = "red";
  textarea.parentNode.insertBefore(errorMsg, textarea.nextSibling);
}

function setupSingleEditor(textarea, toolbarOptions) {
  if (textarea.dataset.quillInitialized === "true") {
    return false;
  }

  try {
    const initialValue = textarea.value;
    const form = textarea.closest("form");
    const editorDiv = createAndReplaceTextarea(textarea);
    const quill = initializeQuillEditor(
      editorDiv,
      toolbarOptions,
      initialValue,
    );
    updateTextareaOnSubmit(form, textarea, quill);
    textarea.dataset.quillInitialized = "true";
    return true;
  } catch (error) {
    handleEditorInitError(textarea, error);
    return false;
  }
}

function initializeEditors() {
  loadQuillStylesheet();

  const textareas = document.getElementsByTagName("textarea");
  if (textareas.length === 0) {
    return;
  }

  const toolbarOptions = getMarkdownToolbarOptions();
  let initializedCount = 0;

  for (const textarea of textareas) {
    if (setupSingleEditor(textarea, toolbarOptions)) {
      initializedCount++;
    }
  }

  if (initializedCount > 0) {
    console.log(
      `Successfully initialized Quill for ${initializedCount} textareas.`,
    );
  }
}

// MDAST conversion functions
function deltaToMdast(delta) {
  const mdast = createRootNode();
  let currentParagraph = null;
  let currentList = null;
  let textBuffer = "";

  for (const op of delta.ops) {
    if (op.delete || op.retain) {
      continue;
    }

    if (typeof op.insert === "string") {
      const text = op.insert;
      const attributes = op.attributes || {};

      if (text === "\n") {
        processLineBreak(
          mdast,
          currentParagraph,
          attributes,
          textBuffer,
          currentList,
        );
        if (
          !attributes.list &&
          !attributes.blockquote &&
          !attributes["code-block"] &&
          !attributes.header
        ) {
          currentList = null;
        }

        // Reset paragraph and buffer after processing line break
        currentParagraph = null;
        textBuffer = "";
        continue;
      }

      // Process regular text
      const node = createTextNode(text, attributes);

      if (!currentParagraph) {
        currentParagraph = createParagraphNode();
      }

      textBuffer += text;
      currentParagraph.children.push(node);
    } else if (isImageInsert(op)) {
      if (!currentParagraph) {
        currentParagraph = createParagraphNode();
      }
      currentParagraph.children.push(createImageNode(op));
    }
  }

  if (currentParagraph) {
    mdast.children.push(currentParagraph);
  }

  return mdast;
}

function createRootNode() {
  return {
    type: "root",
    children: [],
  };
}

function createParagraphNode() {
  return {
    type: "paragraph",
    children: [],
  };
}

function isImageInsert(op) {
  return typeof op.insert === "object" && op.insert.image;
}

function createImageNode(op) {
  return {
    type: "image",
    url: op.insert.image,
    title: op.attributes?.alt || "",
    alt: op.attributes?.alt || "",
  };
}

function createTextNode(text, attributes) {
  let node = {
    type: "text",
    value: text,
  };

  if (attributes.bold) {
    node = wrapNodeWith(node, "strong");
  }

  if (attributes.italic) {
    node = wrapNodeWith(node, "emphasis");
  }

  if (attributes.link) {
    node = {
      type: "link",
      url: attributes.link,
      children: [node],
    };
  }

  return node;
}

function wrapNodeWith(node, type) {
  return {
    type: type,
    children: [node],
  };
}

function processLineBreak(
  mdast,
  currentParagraph,
  attributes,
  textBuffer,
  currentList,
) {
  if (!currentParagraph) {
    handleEmptyLineWithAttributes(mdast, attributes, currentList);
    return;
  }

  if (attributes.header) {
    processHeaderLineBreak(mdast, textBuffer, attributes);
  } else if (attributes["code-block"]) {
    processCodeBlockLineBreak(mdast, textBuffer, attributes);
  } else if (attributes.list) {
    processListLineBreak(mdast, currentParagraph, attributes, currentList);
  } else if (attributes.blockquote) {
    processBlockquoteLineBreak(mdast, currentParagraph);
  } else {
    mdast.children.push(currentParagraph);
  }
}

function handleEmptyLineWithAttributes(mdast, attributes, currentList) {
  if (attributes["code-block"]) {
    mdast.children.push(createEmptyCodeBlock(attributes));
  } else if (attributes.list) {
    const list = ensureList(mdast, attributes, currentList);
    list.children.push(createEmptyListItem());
  } else if (attributes.blockquote) {
    mdast.children.push(createEmptyBlockquote());
  }
}

function createEmptyCodeBlock(attributes) {
  return {
    type: "code",
    value: "",
    lang:
      attributes["code-block"] === "plain" ? null : attributes["code-block"],
  };
}

function createEmptyListItem() {
  return {
    type: "listItem",
    spread: false,
    children: [{ type: "paragraph", children: [] }],
  };
}

function createEmptyBlockquote() {
  return {
    type: "blockquote",
    children: [{ type: "paragraph", children: [] }],
  };
}

function processHeaderLineBreak(mdast, textBuffer, attributes) {
  const lines = textBuffer.split("\n");

  if (lines.length > 1) {
    const lastLine = lines.pop();
    const previousLines = lines.join("\n");

    if (previousLines) {
      mdast.children.push({
        type: "paragraph",
        children: [{ type: "text", value: previousLines }],
      });
    }

    mdast.children.push({
      type: "heading",
      depth: attributes.header,
      children: [{ type: "text", value: lastLine }],
    });
  } else {
    mdast.children.push({
      type: "heading",
      depth: attributes.header,
      children: [{ type: "text", value: textBuffer }],
    });
  }
}

function processCodeBlockLineBreak(mdast, textBuffer, attributes) {
  mdast.children.push({
    type: "code",
    value: textBuffer,
    lang:
      attributes["code-block"] === "plain" ? null : attributes["code-block"],
  });
}

function ensureList(mdast, attributes, currentList) {
  if (!currentList || currentList.ordered !== (attributes.list === "ordered")) {
    const newList = {
      type: "list",
      ordered: attributes.list === "ordered",
      spread: false,
      children: [],
    };
    mdast.children.push(newList);
    return newList;
  }
  return currentList;
}

function processListLineBreak(
  mdast,
  currentParagraph,
  attributes,
  currentList,
) {
  const list = ensureList(mdast, attributes, currentList);

  const listItem = {
    type: "listItem",
    spread: false,
    children: [currentParagraph],
  };

  list.children.push(listItem);
}

function processBlockquoteLineBreak(mdast, currentParagraph) {
  mdast.children.push({
    type: "blockquote",
    children: [currentParagraph],
  });
}

// Main execution
document.addEventListener("DOMContentLoaded", initializeEditors);

export { deltaToMdast };
