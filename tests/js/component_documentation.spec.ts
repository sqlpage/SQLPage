import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import test from "node:test";

// Every built-in component is a Handlebars template in sqlpage/templates/, and
// its properties are documented as rows of the `parameter` table of the
// official site, built by the SQL migrations in examples/official-site/.
// Nothing links the two, so they drift: properties get added to a template and
// never documented, and documented properties outlive the code that read them.
//
// This test builds the official site's documentation database, reads the
// property names out of the templates, and asserts that the two agree.

const require = createRequire(import.meta.url);
const REPO_ROOT = path.resolve(
  path.dirname(require.resolve("../../package.json")),
);
const TEMPLATES_DIR = path.join(REPO_ROOT, "sqlpage", "templates");
const MIGRATIONS_DIR = path.join(
  REPO_ROOT,
  "examples",
  "official-site",
  "sqlpage",
  "migrations",
);

/** Components rendered by src/render.rs rather than by a Handlebars template. */
const COMPONENTS_WITHOUT_A_TEMPLATE = new Set([
  "authentication",
  "cookie",
  "download",
  "dynamic",
  "http_header",
  "json",
  "log",
  "redirect",
  "status_code",
]);

/**
 * Templates that are not user-invocable components, and so have no row in the
 * `component` table: `error` is rendered by SQLPage when a query fails,
 * `default` is the fallback for an unknown component name, and `shell-empty`
 * is selected through the `shell-empty` component rather than documented as
 * one.
 */
const TEMPLATES_WITHOUT_DOCUMENTATION = new Set([
  "default",
  "error",
  "shell-empty",
]);

/**
 * Documented properties that a template cannot be expected to mention, with
 * the reason why. Keep this list short: an entry here is documentation that
 * nothing verifies.
 */
const DOCUMENTED_WITHOUT_A_TEMPLATE_REFERENCE = new Map([
  [
    "shell.target",
    // `target` belongs to the json objects passed as `menu_item`, and the
    // `parameter` table has no level for the sub-properties of a property.
    // It is documented at the top level, and says so in its description.
    "documented as a menu_item sub-property",
  ],
]);

/** Handlebars built-ins plus the helpers registered in src/template_helpers.rs. */
const HELPERS = new Set([
  // handlebars-rust built-ins
  "and",
  "each",
  "eq",
  "gt",
  "gte",
  "if",
  "len",
  "log",
  "lookup",
  "lt",
  "lte",
  "ne",
  "not",
  "or",
  "raw",
  "unless",
  "with",
  // SQLPage helpers
  "all",
  "any",
  "app_config",
  "array_contains",
  "array_contains_case_insensitive",
  "buildinfo",
  "csv_escape",
  "default",
  "delay",
  "entries",
  "flush_delayed",
  "icon_img",
  "loose_eq",
  "markdown",
  "minus",
  "parse_json",
  "plus",
  "replace",
  "rfc2822_date",
  "starts_with",
  "static_path",
  "stringify",
  "sum",
  "to_array",
  "typeof",
  "url_encode",
]);

const LITERALS = new Set([
  "else",
  "true",
  "false",
  "null",
  "NULL",
  "undefined",
]);

type Level = "top" | "row";
type TemplateProperties = { top: Set<string>; row: Set<string> };

/**
 * The context a Handlebars expression is evaluated in. `top` is the first row
 * of the result set, `row` is one of the following rows, and `nested` is an
 * object reached through `{{#each}}` or `{{#with}}` — the sub-properties of a
 * json property, which are documented in the parent property's description
 * rather than as rows of their own.
 */
type Context = "top" | "row" | "nested";

function properties_of_template(source: string): TemplateProperties {
  const top = new Set<string>();
  const row = new Set<string>();
  const contexts: Context[] = ["top"];

  for (const [, raw_body] of source.matchAll(/\{\{([^}]*)\}\}/g)) {
    let body = raw_body
      .replace(/^[{~]+/, "")
      .replace(/[~}]+$/, "")
      .trim();
    if (body.startsWith("!")) continue; // comment

    if (/^#each_row\b/.test(body)) {
      contexts.push("row");
      continue;
    }
    if (/^\/each_row\b/.test(body)) {
      contexts.pop();
      continue;
    }

    const is_closing = body.startsWith("/");
    const opens_a_nested_context = /^#(each|with)\b/.test(body);
    body = body.replace(/^[#^/>&]+/, "").trim();
    if (is_closing) {
      if (/^(each|with)\b/.test(body)) contexts.pop();
      continue;
    }

    // `{{else if (...)}}` chains an inner `if`, which is not a property.
    body = body.replace(/^else\s+/, "");
    // String literals may contain anything, including what looks like a path.
    body = body.replace(/"[^"]*"/g, " ").replace(/'[^']*'/g, " ");
    // Block parameters (`{{#each x as |y|}}`) name the inner context.
    const block_params = body.indexOf(" as |");
    if (block_params >= 0) body = body.slice(0, block_params);

    // A bare name is a helper call when it is the first token of the mustache
    // or of a subexpression, and a property reference anywhere else.
    let is_callee = true;
    for (const token of body.split(/(\s+|\(|\))/)) {
      if (!token.trim()) continue;
      if (token === "(") {
        is_callee = true;
        continue;
      }
      if (token === ")") {
        is_callee = false;
        continue;
      }
      const in_callee_position = is_callee;
      is_callee = false;

      let name = token;
      if (/^[0-9]/.test(name)) continue; // numeric literal
      if (name.includes("=")) name = name.slice(name.indexOf("=") + 1); // hash argument
      let parents = 0;
      while (name.startsWith("../")) {
        parents += 1;
        name = name.slice(3);
      }
      if (name.startsWith("@") || name.startsWith(".") || name.startsWith("|"))
        continue;
      if (name.startsWith("this.")) name = name.slice(5);
      if (name === "this" || name === "") continue;
      name = name.split(/[.[]/)[0]; // `a.b` and `a.[0]` are reads of `a`
      if (!/^[A-Za-z_][A-Za-z_0-9]*$/.test(name)) continue;
      if (LITERALS.has(name)) continue;
      if (parents === 0 && in_callee_position && HELPERS.has(name)) continue;

      const context = contexts[Math.max(0, contexts.length - 1 - parents)];
      if (context === "top") top.add(name);
      else if (context === "row") row.add(name);
    }

    if (opens_a_nested_context) contexts.push("nested");
  }

  return { top, row };
}

function read_templates(): Map<string, TemplateProperties> {
  const templates = new Map<string, TemplateProperties>();
  for (const file of fs.readdirSync(TEMPLATES_DIR)) {
    if (!file.endsWith(".handlebars")) continue;
    const name = file.slice(0, -".handlebars".length);
    if (TEMPLATES_WITHOUT_DOCUMENTATION.has(name)) continue;
    const source = fs.readFileSync(path.join(TEMPLATES_DIR, file), "utf8");
    templates.set(name, properties_of_template(source));
  }
  return templates;
}

function read_documentation(): Map<string, TemplateProperties> {
  const migrations = fs
    .readdirSync(MIGRATIONS_DIR)
    .filter((file) => file.endsWith(".sql"))
    // SQLPage applies migrations in the order of their numeric prefix.
    .sort(
      (a, b) =>
        Number.parseInt(a, 10) - Number.parseInt(b, 10) || a.localeCompare(b),
    );

  const db = new DatabaseSync(":memory:");
  for (const file of migrations) {
    db.exec(fs.readFileSync(path.join(MIGRATIONS_DIR, file), "utf8"));
  }

  const documented = new Map<string, TemplateProperties>();
  const rows = db
    .prepare("select component, name, top_level from parameter")
    .all() as { component: string; name: string; top_level: number }[];
  for (const { component, name, top_level } of rows) {
    let entry = documented.get(component);
    if (!entry) {
      entry = { top: new Set(), row: new Set() };
      documented.set(component, entry);
    }
    (top_level ? entry.top : entry.row).add(name);
  }
  db.close();
  return documented;
}

const templates = read_templates();
const documented = read_documentation();

test("every component documented by the official site has a template", () => {
  const missing = [...documented.keys()].filter(
    (component) =>
      !templates.has(component) &&
      !COMPONENTS_WITHOUT_A_TEMPLATE.has(component),
  );
  assert.deepEqual(
    missing,
    [],
    `These components are documented but have no sqlpage/templates/*.handlebars file. ` +
      `If they are rendered by src/render.rs, add them to COMPONENTS_WITHOUT_A_TEMPLATE.`,
  );
});

test("every property read by a template is documented", () => {
  const undocumented: string[] = [];
  for (const [component, properties] of templates) {
    const docs = documented.get(component) ?? {
      top: new Set(),
      row: new Set(),
    };
    for (const level of ["top", "row"] satisfies Level[]) {
      for (const name of properties[level]) {
        if (!docs[level].has(name))
          undocumented.push(`${component}.${name} (${level}-level)`);
      }
    }
  }
  assert.deepEqual(
    undocumented.sort(),
    [],
    `These properties are read by a template but not documented. Add a row to ` +
      `the component's migration in examples/official-site/sqlpage/migrations/.`,
  );
});

test("every documented property is read by its template", () => {
  const unimplemented: string[] = [];
  for (const [component, docs] of documented) {
    const properties = templates.get(component);
    if (!properties) continue; // rendered by src/, checked by the test above
    for (const level of ["top", "row"] satisfies Level[]) {
      for (const name of docs[level]) {
        if (properties[level].has(name)) continue;
        if (DOCUMENTED_WITHOUT_A_TEMPLATE_REFERENCE.has(`${component}.${name}`))
          continue;
        const other_level = level === "top" ? "row" : "top";
        unimplemented.push(
          properties[other_level].has(name)
            ? `${component}.${name} is documented as ${level}-level but the template reads it at the ${other_level} level`
            : `${component}.${name} (${level}-level) is documented but no template reads it`,
        );
      }
    }
  }
  assert.deepEqual(
    unimplemented.sort(),
    [],
    `Remove the documentation, or fix its top_level, in ` +
      `examples/official-site/sqlpage/migrations/.`,
  );
});
