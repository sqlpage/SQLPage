import Quill from "https://esm.sh/quill@2.0.3";
import { toMarkdown as mdastUtilToMarkdown } from "https://esm.sh/mdast-util-to-markdown@2.1.2";

/**
 * @typedef {Object} QuillAttributes
 * @property {boolean} [bold] - Whether the text is bold.
 * @property {boolean} [italic] - Whether the text is italic.
 * @property {string} [link] - URL if the text is a link.
 * @property {number} [header] - Header level (1-3).
 * @property {string} [list] - List type ('ordered' or 'bullet').
 * @property {boolean} [blockquote] - Whether the text is in a blockquote.
 * @property {string} [code-block] - Code language if in a code block.
 * @property {string} [alt] - Alt text for images.
 */

/**
 * @typedef {Object} QuillOperation
 * @property {string|Object} [insert] - Content to insert (string or object with image URL).
 * @property {number} [delete] - Number of characters to delete.
 * @property {number} [retain] - Number of characters to retain.
 * @property {QuillAttributes} [attributes] - Formatting attributes.
 */

/**
 * @typedef {Object} QuillDelta
 * @property {Array<QuillOperation>} ops - Array of operations in the delta.
 */

/**
 * Converts Quill Delta object to a Markdown string using mdast.
 * @param {QuillDelta} delta - Quill Delta object (https://quilljs.com/docs/delta/).
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

/**
 * Extracts plain text from a Quill Delta object.
 * @param {QuillDelta} delta - Quill Delta object.
 * @returns {string} - Plain text extracted from the delta.
 */
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
 * @returns {Array<Array<any>>} - Quill toolbar options.
 */
function getMarkdownToolbarOptions() {
  return [
    [{ header: 1 }, { header: 2 }, { header: 3 }],
    ["bold", "italic", "code"],
    ["link", "image", "blockquote", "code-block"],
    [{ list: "ordered" }, { list: "bullet" }],
    ["clean"],
  ];
}

/**
 * Initializes a Quill editor instance on a given div.
 * @param {HTMLDivElement} editorDiv - The div element for the editor.
 * @param {Array<Array<any>>} toolbarOptions - The toolbar configuration.
 * @param {string} initialValue - The initial content for the editor.
 * @returns {Quill} - The initialized Quill instance.
 */
function initializeQuillEditor(editorDiv, toolbarOptions, initialValue) {
  const quill = new Quill(editorDiv, {
    theme: "snow",
    modules: {
      toolbar: toolbarOptions,
    },
    formats: [
      "bold",
      "italic",
      "link",
      "header",
      "list",
      "blockquote",
      "code",
      "code-block",
      "image",
    ],
  });
  if (initialValue) {
    quill.setText(initialValue);
  }
  return quill;
}

/**
 * Attaches a submit event listener to the form to update the hidden textarea.
 * @param {HTMLFormElement|null} form - The form containing the editor.
 * @param {HTMLTextAreaElement} textarea - The original (hidden) textarea.
 * @param {Quill} quill - The Quill editor instance.
 * @returns {void}
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

/**
 * Loads the Quill CSS stylesheet.
 * @returns {void}
 */
function loadQuillStylesheet() {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = "https://esm.sh/quill@2.0.3/dist/quill.snow.css";
  document.head.appendChild(link);
}

/**
 * Handles errors during editor initialization.
 * @param {HTMLTextAreaElement} textarea - The textarea that failed initialization.
 * @param {Error} error - The error that occurred.
 * @returns {void}
 */
function handleEditorInitError(textarea, error) {
  console.error("Failed to initialize Quill for textarea:", textarea, error);
  textarea.style.display = "";
  const errorMsg = document.createElement("p");
  errorMsg.textContent = "Failed to load rich text editor.";
  errorMsg.style.color = "red";
  textarea.parentNode.insertBefore(errorMsg, textarea.nextSibling);
}

/**
 * Sets up a single editor for a textarea.
 * @param {HTMLTextAreaElement} textarea - The textarea to replace with an editor.
 * @param {Array<Array<any>>} toolbarOptions - The toolbar configuration.
 * @returns {boolean} - Whether the setup was successful.
 */
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

/**
 * Initializes Quill editors for all textareas in the document.
 * @returns {void}
 */
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
/**
 * @typedef {Object} MdastNode
 * @property {string} type - The type of the node.
 * @property {Array<MdastNode>} [children] - Child nodes.
 * @property {string} [value] - Text value for text nodes.
 * @property {string} [url] - URL for link and image nodes.
 * @property {string} [title] - Title for image nodes.
 * @property {string} [alt] - Alt text for image nodes.
 * @property {number} [depth] - Depth for heading nodes.
 * @property {boolean} [ordered] - Whether the list is ordered.
 * @property {boolean} [spread] - Whether the list is spread.
 * @property {string} [lang] - Language for code blocks.
 */

/**
 * Converts a Quill Delta to a MDAST (Markdown Abstract Syntax Tree).
 * @param {QuillDelta} delta - The Quill Delta to convert.
 * @returns {MdastNode} - The root MDAST node.
 */
function deltaToMdast(delta) {
  const mdast = createRootNode();
  /** @type {MdastNode|null} */
  let currentParagraph = null;
  /** @type {MdastNode|null} */
  let currentList = null;
  let textBuffer = "";

  for (const op of delta.ops) {
    if (isImageInsert(op)) {
      if (!currentParagraph) {
        currentParagraph = createParagraphNode();
      }
      currentParagraph.children.push(createImageNode(op));
    }
    if (typeof op.insert !== "string") continue;

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
  }

  if (currentParagraph) {
    mdast.children.push(currentParagraph);
  }

  return mdast;
}

/**
 * Creates a root MDAST node.
 * @returns {MdastNode} - The root node.
 */
function createRootNode() {
  return {
    type: "root",
    children: [],
  };
}

/**
 * Creates a paragraph MDAST node.
 * @returns {MdastNode} - The paragraph node.
 */
function createParagraphNode() {
  return {
    type: "paragraph",
    children: [],
  };
}

/**
 * Checks if an operation is an image insertion.
 * @param {Object} op - The operation to check.
 * @returns {boolean} - Whether the operation is an image insertion.
 */
function isImageInsert(op) {
  return typeof op.insert === "object" && op.insert.image;
}

/**
 * Creates an image MDAST node.
 * @param {Object} op - The operation containing the image.
 * @returns {MdastNode} - The image node.
 */
function createImageNode(op) {
  return {
    type: "image",
    url: op.insert.image,
    title: op.attributes?.alt || "",
    alt: op.attributes?.alt || "",
  };
}

/**
 * Creates a text MDAST node with optional formatting.
 * @param {string} text - The text content.
 * @param {Object} attributes - The formatting attributes.
 * @returns {MdastNode} - The formatted text node.
 */
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

/**
 * Wraps a node with a formatting container.
 * @param {MdastNode} node - The node to wrap.
 * @param {string} type - The type of container.
 * @returns {MdastNode} - The wrapped node.
 */
function wrapNodeWith(node, type) {
  return {
    type: type,
    children: [node],
  };
}

/**
 * Processes a line break in the Delta.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {MdastNode|null} currentParagraph - The current paragraph being built.
 * @param {Object} attributes - The attributes for the line.
 * @param {string} textBuffer - The text buffer for the current line.
 * @param {MdastNode|null} currentList - The current list being built.
 * @returns {void}
 */
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

/**
 * Handles an empty line with special attributes.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {Object} attributes - The attributes for the line.
 * @param {MdastNode|null} currentList - The current list being built.
 * @returns {void}
 */
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

/**
 * Creates an empty code block MDAST node.
 * @param {Object} attributes - The attributes for the code block.
 * @returns {MdastNode} - The code block node.
 */
function createEmptyCodeBlock(attributes) {
  return {
    type: "code",
    value: "",
    lang:
      attributes["code-block"] === "plain" ? null : attributes["code-block"],
  };
}

/**
 * Creates an empty list item MDAST node.
 * @returns {MdastNode} - The list item node.
 */
function createEmptyListItem() {
  return {
    type: "listItem",
    spread: false,
    children: [{ type: "paragraph", children: [] }],
  };
}

/**
 * Creates an empty blockquote MDAST node.
 * @returns {MdastNode} - The blockquote node.
 */
function createEmptyBlockquote() {
  return {
    type: "blockquote",
    children: [{ type: "paragraph", children: [] }],
  };
}

/**
 * Processes a header line break.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {string} textBuffer - The text buffer for the current line.
 * @param {Object} attributes - The attributes for the line.
 * @returns {void}
 */
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

/**
 * Processes a code block line break.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {string} textBuffer - The text buffer for the current line.
 * @param {Object} attributes - The attributes for the line.
 * @returns {void}
 */
function processCodeBlockLineBreak(mdast, textBuffer, attributes) {
  const lang =
    attributes["code-block"] === "plain" ? null : attributes["code-block"];
  // Two code blocks in a row are merged into one
  const lastChild = mdast.children[mdast.children.length - 1];
  if (lastChild && lastChild.type === "code" && lastChild.lang === lang) {
    lastChild.value += `\n${textBuffer}`;
  } else {
    mdast.children.push({
      type: "code",
      value: textBuffer,
      lang,
    });
  }
}

/**
 * Ensures a list exists in the MDAST.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {Object} attributes - The attributes for the line.
 * @param {MdastNode|null} currentList - The current list being built.
 * @returns {MdastNode} - The list node.
 */
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

/**
 * Processes a list line break.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {MdastNode} currentParagraph - The current paragraph being built.
 * @param {Object} attributes - The attributes for the line.
 * @param {MdastNode|null} currentList - The current list being built.
 * @returns {void}
 */
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

/**
 * Processes a blockquote line break.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {MdastNode} currentParagraph - The current paragraph being built.
 * @returns {void}
 */
function processBlockquoteLineBreak(mdast, currentParagraph) {
  mdast.children.push({
    type: "blockquote",
    children: [currentParagraph],
  });
}

// Main execution
document.addEventListener("DOMContentLoaded", initializeEditors);
