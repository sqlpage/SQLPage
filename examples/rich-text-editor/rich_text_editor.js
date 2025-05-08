import { fromMarkdown } from "https://esm.sh/mdast-util-from-markdown@2.0.0";
import { toMarkdown as mdastUtilToMarkdown } from "https://esm.sh/mdast-util-to-markdown@2.1.2";
import Quill from "https://esm.sh/quill@2.0.3";

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
  // Hide the original textarea, but keep it focusable for validation
  textarea.style = "transform: scale(0); position: absolute; opacity: 0;";
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
    const delta = markdownToDelta(initialValue);
    quill.setContents(delta);
  }
  return quill;
}

/**
 * Converts Markdown string to a Quill Delta object.
 * @param {string} markdown - The markdown string to convert.
 * @returns {QuillDelta} - Quill Delta representation.
 */
function markdownToDelta(markdown) {
  try {
    const mdastTree = fromMarkdown(markdown);
    return mdastToDelta(mdastTree);
  } catch (error) {
    console.error("Error parsing markdown:", error);
    return { ops: [{ insert: markdown }] };
  }
}

/**
 * Converts MDAST to Quill Delta.
 * @param {MdastNode} tree - The MDAST tree to convert.
 * @returns {QuillDelta} - Quill Delta representation.
 */
function mdastToDelta(tree) {
  const delta = { ops: [] };
  if (!tree || !tree.children) return delta;

  for (const node of tree.children) {
    traverseMdastNode(node, delta);
  }

  return delta;
}

/**
 * Recursively traverse MDAST nodes and convert to Delta operations.
 * @param {MdastNode} node - The MDAST node to process.
 * @param {QuillDelta} delta - The Delta object to append operations to.
 * @param {QuillAttributes} [attributes={}] - The current attributes to apply.
 */
function traverseMdastNode(node, delta, attributes = {}) {
  if (!node) return;

  switch (node.type) {
    case "root":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta);
      }
      break;

    case "paragraph":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, attributes);
      }
      delta.ops.push({ insert: "\n" });
      break;

    case "heading":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, { header: node.depth });
      }
      delta.ops.push({ insert: "\n", attributes: { header: node.depth } });
      break;

    case "text":
      delta.ops.push({ insert: node.value || "", attributes });
      break;

    case "strong":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, { ...attributes, bold: true });
      }
      break;

    case "emphasis":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, { ...attributes, italic: true });
      }
      break;

    case "link":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, { ...attributes, link: node.url });
      }
      break;

    case "image":
      delta.ops.push({
        insert: { image: node.url },
        attributes: { alt: node.alt || "" },
      });
      break;

    case "list":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, {
          ...attributes,
          list: node.ordered ? "ordered" : "bullet",
        });
      }
      break;

    case "listItem":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, attributes);
      }
      break;

    case "blockquote":
      for (const child of node.children || []) {
        traverseMdastNode(child, delta, { ...attributes, blockquote: true });
      }
      break;

    case "code":
      delta.ops.push({
        insert: node.value || "",
        attributes: { "code-block": node.lang || "plain" },
      });
      delta.ops.push({
        insert: "\n",
        attributes: { "code-block": node.lang || "plain" },
      });
      break;

    case "inlineCode":
      delta.ops.push({ insert: node.value || "", attributes: { code: true } });
      break;

    default:
      if (node.children) {
        for (const child of node.children) {
          traverseMdastNode(child, delta, attributes);
        }
      } else if (node.value) {
        delta.ops.push({ insert: node.value, attributes });
      }
  }
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
  form.addEventListener("submit", (event) => {
    const delta = quill.getContents();
    const markdownContent = deltaToMarkdown(delta);
    textarea.value = markdownContent;
    if (textarea.required && !markdownContent) {
      textarea.setCustomValidity(`${textarea.name} cannot be empty`);
      quill.once("text-change", (delta) => {
        textarea.value = deltaToMarkdown(delta);
        textarea.setCustomValidity("");
      });
      quill.focus();
      event.preventDefault();
    }
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

    // Handle newlines within text content
    if (text.includes("\n") && text !== "\n") {
      const lines = text.split("\n");

      // Process all lines except the last one as complete lines
      for (let i = 0; i < lines.length - 1; i++) {
        const line = lines[i];
        if (line.length > 0) {
          // Add text to current paragraph
          if (!currentParagraph) {
            currentParagraph = createParagraphNode();
          }
          const nodes = createTextNodes(line, attributes);
          currentParagraph.children.push(...nodes);
          textBuffer = line;
        }

        // Process line break with empty attributes (regular paragraph break)
        processLineBreak(mdast, currentParagraph, {}, textBuffer, currentList);
        currentParagraph = null;
        textBuffer = "";
      }

      // Add the last line to the buffer without processing the line break yet
      const lastLine = lines[lines.length - 1];
      if (lastLine.length > 0) {
        if (!currentParagraph) {
          currentParagraph = createParagraphNode();
        }
        const nodes = createTextNodes(lastLine, attributes);
        currentParagraph.children.push(...nodes);
        textBuffer = lastLine;
      }

      continue;
    }

    if (text === "\n") {
      currentList = processLineBreak(
        mdast,
        currentParagraph,
        attributes,
        textBuffer,
        currentList,
      );

      // Reset paragraph and buffer after processing line break
      currentParagraph = null;
      textBuffer = "";
      continue;
    }

    // Process regular text
    const nodes = createTextNodes(text, attributes);

    if (!currentParagraph) {
      currentParagraph = createParagraphNode();
    }

    textBuffer += text;
    currentParagraph.children.push(...nodes);
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
 * @returns {MdastNode[]} - The formatted text nodes.
 */
function createTextNodes(text, attributes) {
  let nodes = text.split("\n").flatMap((value, i) => [
    ...(i > 0 ? [{ type: "break" }] : []),
    {
      type: "text",
      value,
    },
  ]);

  if (attributes.bold) {
    nodes = [wrapNodesWith(nodes, "strong")];
  }

  if (attributes.italic) {
    nodes = [wrapNodesWith(nodes, "emphasis")];
  }

  if (attributes.link) {
    nodes = [{ ...wrapNodesWith(nodes, "link"), url: attributes.link }];
  }

  return nodes;
}

/**
 * Wraps a node with a formatting container.
 * @param {MdastNode[]} children - The node to wrap.
 * @param {string} type - The type of container.
 * @returns {MdastNode} - The wrapped node.
 */
function wrapNodesWith(children, type) {
  return {
    type: type,
    children,
  };
}

/**
 * Processes a line break in the Delta.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {MdastNode|null} currentParagraph - The current paragraph being built.
 * @param {Object} attributes - The attributes for the line.
 * @param {string} textBuffer - The text buffer for the current line.
 * @param {MdastNode|null} currentList - The current list being built.
 * @returns {MdastNode|null} - The updated current list.
 */
function processLineBreak(
  mdast,
  currentParagraph,
  attributes,
  textBuffer,
  currentList,
) {
  if (!currentParagraph) {
    return handleEmptyLineWithAttributes(mdast, attributes, currentList);
  }

  if (attributes.header) {
    processHeaderLineBreak(mdast, textBuffer, attributes);
    return null;
  }

  if (attributes["code-block"]) {
    processCodeBlockLineBreak(mdast, textBuffer, attributes);
    return currentList;
  }

  if (attributes.list) {
    return processListLineBreak(
      mdast,
      currentParagraph,
      attributes,
      currentList,
    );
  }

  if (attributes.blockquote) {
    processBlockquoteLineBreak(mdast, currentParagraph);
    return currentList;
  }

  // Default case: regular paragraph
  mdast.children.push(currentParagraph);
  return null;
}

/**
 * Handles an empty line with special attributes.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {Object} attributes - The attributes for the line.
 * @param {MdastNode|null} currentList - The current list being built.
 * @returns {MdastNode|null} - The updated current list.
 */
function handleEmptyLineWithAttributes(mdast, attributes, currentList) {
  if (attributes["code-block"]) {
    mdast.children.push(createEmptyCodeBlock(attributes));
    return currentList;
  }

  if (attributes.list) {
    const list = ensureList(mdast, attributes, currentList);
    list.children.push(createEmptyListItem());
    return list;
  }

  if (attributes.blockquote) {
    mdast.children.push(createEmptyBlockquote());
    return currentList;
  }

  return null;
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

  // Find the last code block with the same language
  let lastCodeBlock = null;
  for (let i = mdast.children.length - 1; i >= 0; i--) {
    const child = mdast.children[i];
    if (child.type === "code" && child.lang === lang) {
      lastCodeBlock = child;
      break;
    }
  }

  if (lastCodeBlock) {
    // Append to existing code block with same language
    lastCodeBlock.value += `\n${textBuffer}`;
  } else {
    // Create new code block
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
  const isOrderedList = attributes.list === "ordered";

  // If there's no current list or the list type doesn't match
  if (!currentList || currentList.ordered !== isOrderedList) {
    // Check if the last child is a list of the correct type
    const lastChild = mdast.children[mdast.children.length - 1];
    if (
      lastChild &&
      lastChild.type === "list" &&
      lastChild.ordered === isOrderedList
    ) {
      // Use the last list if it matches the type
      return lastChild;
    }

    // Create a new list
    const newList = {
      type: "list",
      ordered: isOrderedList,
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
 * @returns {MdastNode} - The updated list node.
 */
function processListLineBreak(
  mdast,
  currentParagraph,
  attributes,
  currentList,
) {
  const list = ensureList(mdast, attributes, currentList);

  // Check if this list item already exists to avoid duplication
  const paragraphContent = JSON.stringify(currentParagraph.children);
  const isDuplicate = list.children.some(
    (item) =>
      item.children?.length === 1 &&
      JSON.stringify(item.children[0].children) === paragraphContent,
  );

  if (!isDuplicate) {
    const listItem = {
      type: "listItem",
      spread: false,
      children: [currentParagraph],
    };

    list.children.push(listItem);
  }

  return list;
}

/**
 * Processes a blockquote line break.
 * @param {MdastNode} mdast - The root MDAST node.
 * @param {MdastNode} currentParagraph - The current paragraph being built.
 * @returns {void}
 */
function processBlockquoteLineBreak(mdast, currentParagraph) {
  // Look for an existing blockquote with identical content to avoid duplication
  const paragraphContent = JSON.stringify(currentParagraph.children);
  const existingBlockquote = mdast.children.find(
    (child) =>
      child.type === "blockquote" &&
      child.children?.length === 1 &&
      JSON.stringify(child.children[0].children) === paragraphContent,
  );

  if (!existingBlockquote) {
    mdast.children.push({
      type: "blockquote",
      children: [currentParagraph],
    });
  }
}

// Main execution
document.addEventListener("DOMContentLoaded", initializeEditors);
