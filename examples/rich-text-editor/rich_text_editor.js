import Quill from "https://esm.sh/quill@2.0.3";
import { toMarkdown as mdastUtilToMarkdown } from "https://esm.sh/mdast-util-to-markdown@2.1.2";

/**
 * Converts Quill Delta object to a Markdown string using mdast.
 * @param {object} delta - Quill Delta object (https://quilljs.com/docs/delta/).
 * @returns {string} - Markdown representation.
 */
function deltaToMarkdown(delta) {
  try {
    console.log("delta", delta);
    const mdastTree = deltaToMdast(delta); // Convert Delta to MDAST
    console.log("mdastTree", mdastTree);
    const options = {
      bullet: "*",
      listItemIndent: "one",
    };
    const markdown = mdastUtilToMarkdown(mdastTree, options); // Convert MDAST to Markdown
    console.log("markdown", markdown);
    return markdown;
  } catch (error) {
    console.error("Error during Delta to Markdown conversion:", error);
    try {
      return delta.ops
        .map((op) => (typeof op.insert === "string" ? op.insert : ""))
        .join("")
        .trim();
    } catch (e) {
      return "";
    }
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
    [{ header: [1, 2, 3, false] }],
    ["bold", "italic"],
    ["link", "image", "blockquote"],
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
  form.addEventListener("submit", (evt) => {
    const delta = quill.getContents();
    const markdownContent = deltaToMarkdown(delta);
    textarea.value = markdownContent;
    console.log(
      `Converted content for ${textarea.name || "textarea"} to Markdown and updated value.`,
    );
  });
}

const initializeEditors = () => {
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = "https://esm.sh/quill@2.0.3/dist/quill.snow.css";
  document.head.appendChild(link);

  const textareas = document.getElementsByTagName("textarea");
  if (textareas.length === 0) {
    console.log("No textareas found to initialize Quill on.");
    return;
  }

  const markdownToolbarOptions = getMarkdownToolbarOptions();
  let initializedCount = 0;

  for (const textarea of textareas) {
    if (textarea.dataset.quillInitialized === "true") {
      continue;
    }

    try {
      const initialValue = textarea.value;
      const form = textarea.closest("form");
      const editorDiv = createAndReplaceTextarea(textarea);
      const quill = initializeQuillEditor(
        editorDiv,
        markdownToolbarOptions,
        initialValue,
      );
      updateTextareaOnSubmit(form, textarea, quill);
      textarea.dataset.quillInitialized = "true";
      initializedCount++;
    } catch (error) {
      console.error(
        "Failed to initialize Quill for textarea:",
        textarea,
        error,
      );
      textarea.style.display = "";
      const errorMsg = document.createElement("p");
      errorMsg.textContent = "Failed to load rich text editor.";
      errorMsg.style.color = "red";
      textarea.parentNode.insertBefore(errorMsg, textarea.nextSibling);
    }
  }

  if (initializedCount > 0) {
    console.log(
      `Successfully initialized Quill for ${initializedCount} textareas.`,
    );
  } else if (textareas.length > 0) {
    console.log(
      "Found 'rich-text-editor' textareas, but none were newly initialized (already initialized or failed).",
    );
  }
};

function deltaToMdast(delta) {
  const mdast = {
    type: "root",
    children: [],
  };

  let currentParagraph = null;
  let textBuffer = "";

  for (const op of delta.ops) {
    if (op.delete) {
      continue;
    }

    if (op.retain) {
      continue;
    }

    if (typeof op.insert === "string") {
      const text = op.insert;
      const attributes = op.attributes || {};

      if (text === "\n") {
        if (currentParagraph) {
          if (attributes.header) {
            const lines = textBuffer.split("\n");
            if (lines.length > 1) {
              const lastLine = lines.pop();
              const previousLines = lines.join("\n");

              if (previousLines) {
                const paragraph = {
                  type: "paragraph",
                  children: [{ type: "text", value: previousLines }],
                };
                mdast.children.push(paragraph);
              }

              const heading = {
                type: "heading",
                depth: attributes.header,
                children: [{ type: "text", value: lastLine }],
              };
              mdast.children.push(heading);
            } else {
              const heading = {
                type: "heading",
                depth: attributes.header,
                children: [{ type: "text", value: textBuffer }],
              };
              mdast.children.push(heading);
            }
            currentParagraph = null;
            textBuffer = "";
          } else {
            mdast.children.push(currentParagraph);
            currentParagraph = null;
          }
        }
        continue;
      }

      textBuffer += text;
      let node = {
        type: "text",
        value: text,
      };

      if (attributes.bold) {
        node = {
          type: "strong",
          children: [node],
        };
      }

      if (attributes.italic) {
        node = {
          type: "emphasis",
          children: [node],
        };
      }

      if (attributes.link) {
        node = {
          type: "link",
          url: attributes.link,
          children: [node],
        };
      }

      if (!currentParagraph) {
        currentParagraph = {
          type: "paragraph",
          children: [],
        };
      }

      currentParagraph.children.push(node);
    } else if (typeof op.insert === "object") {
      if (op.insert.image) {
        const imageNode = {
          type: "image",
          url: op.insert.image,
          title: op.attributes?.alt || "",
          alt: op.attributes?.alt || "",
        };

        if (!currentParagraph) {
          currentParagraph = {
            type: "paragraph",
            children: [],
          };
        }

        currentParagraph.children.push(imageNode);
        textBuffer = "";
      }
    }
  }

  if (currentParagraph) {
    mdast.children.push(currentParagraph);
  }

  return mdast;
}

// --- Main Script Execution ---
document.addEventListener("DOMContentLoaded", initializeEditors);
