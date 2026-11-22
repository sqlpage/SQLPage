import { existsSync, readdirSync, readFileSync } from "node:fs";
import { basename, join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const defaultContent = fileURLToPath(
  new URL("../examples/official-site/content/", import.meta.url),
);

function read(path) {
  return readFileSync(path, "utf8");
}

function at(path, callback) {
  try {
    return callback();
  } catch (error) {
    throw new Error(`${path}: ${error.message}`, { cause: error });
  }
}

function fields(object, allowed) {
  if (!object || typeof object !== "object" || Array.isArray(object)) {
    throw new Error("Expected a mapping of field names to values");
  }
  for (const key of Object.keys(object)) {
    if (!allowed.includes(key)) throw new Error(`Unknown field "${key}"`);
  }
  return object;
}

function text(value, name) {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`Expected text in "${name}"`);
  }
  return value;
}

function list(value, name) {
  if (!Array.isArray(value)) throw new Error(`Expected a list in "${name}"`);
  return value;
}

function markdownFiles(dir, visit, parseHeader = parse) {
  for (const name of readdirSync(dir)
    .filter((s) => s.endsWith(".md"))
    .sort()) {
    const path = join(dir, name);
    at(path, () => {
      const file = read(path).replaceAll("\r\n", "\n");
      const match =
        file.startsWith("---\n") &&
        /^---\n([\s\S]*?)^---[ \t]*(?:\n|$)/m.exec(file);
      if (file.startsWith("---\n") && !match)
        throw new Error("Unclosed YAML front matter");
      const meta = match ? (parseHeader(match[1]) ?? {}) : {};
      const body = file.slice(match ? match[0].length : 0).trim();
      visit(basename(name, ".md"), meta, body);
    });
  }
}

function exampleHeader(header) {
  const meta = parse(header, {
    mapAsMap: true,
    intAsBigInt: true,
    stringKeys: true,
  });
  return meta instanceof Map ? Object.fromEntries(meta) : meta;
}

// A JS object would reorder numeric column names, and a JS Number would
// round 64-bit integers. The example's JSON determines the SQL shown on the site.
function exampleJson(value) {
  if (Array.isArray(value)) return `[${value.map(exampleJson).join(",")}]`;
  if (value instanceof Map) {
    return `{${Array.from(value, ([key, v]) => `${JSON.stringify(key)}:${exampleJson(v)}`).join(",")}}`;
  }
  if (typeof value === "bigint") return value.toString();
  if (typeof value === "number" && !Number.isFinite(value))
    throw new Error("Example numbers must be finite");
  return JSON.stringify(value);
}

function schemaType(schema) {
  if (schema.enum)
    return schema.enum.map((value) => JSON.stringify(value)).join(" | ");
  if (schema.anyOf) return schema.anyOf.map(schemaType).join(" or ");
  const types = Array.isArray(schema.type) ? schema.type : [schema.type];
  return types
    .map((type) => {
      if (type === "array") return `${schemaType(schema.items)}[]`;
      if (type === "integer" || type === "number") {
        if (schema.minimum !== undefined && schema.maximum !== undefined)
          return `${type} (${schema.minimum} to ${schema.maximum})`;
        if (schema.minimum !== undefined)
          return `${type} (≥ ${schema.minimum})`;
        if (schema.maximum !== undefined)
          return `${type} (≤ ${schema.maximum})`;
      }
      return text(type, "type");
    })
    .join(" or ");
}

function configDefault(schema) {
  if (schema["x-default-description"]) return schema["x-default-description"];
  if (!Object.hasOwn(schema, "default")) return null;
  const value = JSON.stringify(schema.default);
  const longestTick = Math.max(
    0,
    ...(value.match(/`+/g) ?? []).map((s) => s.length),
  );
  const delimiter = "`".repeat(longestTick + 1);
  return `${delimiter}${value}${delimiter}`;
}

export function buildDocumentationSeed(schema, content = defaultContent) {
  const db = new DatabaseSync(":memory:");
  const statements = [];

  function sqlFile(name) {
    const path = join(import.meta.dirname, "site", name);
    at(path, () => {
      const sql = read(path);
      db.exec(sql);
      statements.push(sql);
    });
  }

  function insert(table, record) {
    const columns = Object.keys(record).map((name) => `"${name}"`);
    const values = Object.values(record).map((v) => v ?? null);
    if (values.some((v) => typeof v === "string" && v.includes("\0"))) {
      throw new Error(`NUL character in ${table}`);
    }
    const query = db.prepare(
      `INSERT INTO ${table} (${columns.join(", ")}) VALUES (${values.map(() => "?").join(", ")});`,
    );
    query.run(...values);
    // SQLite quotes the exact same values for SQLPage's startup migration.
    statements.push(query.expandedSQL);
  }

  function collection(name, table, columns) {
    const path = join(content, `${name}.yaml`);
    at(path, () => {
      for (const record of list(parse(read(path)), name)) {
        fields(record, columns);
        for (const column of columns) {
          if (!(column in record)) throw new Error(`Missing "${column}"`);
        }
        insert(table, record);
      }
    });
  }

  function parameter(record) {
    fields(record, ["name", "type", "description", "required"]);
    if (record.required !== undefined && typeof record.required !== "boolean") {
      throw new Error(`Expected a boolean in "${record.name}.required"`);
    }
    return {
      name: text(record.name, "name"),
      type: text(record.type, "type"),
      description_md: text(record.description, "description"),
      optional: record.required ? 0 : 1,
    };
  }

  function references(kind, shared) {
    const directory = join(content, kind);
    const component = kind === "components";
    const extra = component ? ["top", "row"] : ["parameters", "return_type"];
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (
        entry.isDirectory() &&
        !existsSync(join(directory, `${entry.name}.md`))
      ) {
        throw new Error(
          `${join(directory, entry.name)} has no corresponding reference ${entry.name}.md`,
        );
      }
    }
    markdownFiles(directory, (name, meta, body) => {
      fields(meta, ["icon", "since", "category", "summary", ...extra]);
      const common = {
        name,
        icon: meta.icon,
        introduced_in_version: meta.since,
        catalog_category: meta.category,
        catalog_summary: meta.summary,
      };
      if (!component) {
        insert("sqlpage_functions", {
          ...common,
          description_md: body,
          return_type: meta.return_type,
        });
        for (const [i, p] of list(
          meta.parameters ?? [],
          "parameters",
        ).entries()) {
          insert("sqlpage_function_parameters", {
            function: name,
            index: i + 1,
            ...parameter(p),
          });
        }
        return;
      }
      insert("component", { ...common, description: body });
      for (const level of ["top", "row"]) {
        for (let p of list(meta[level] ?? [], level)) {
          if (typeof p === "string") {
            if (!Object.hasOwn(shared, p))
              throw new Error(`Unknown shared parameter "${p}"`);
            p = shared[p];
          }
          insert("parameter", {
            component: name,
            top_level: level === "top" ? 1 : 0,
            ...parameter(p),
          });
        }
      }
      const examples = join(directory, name, "examples");
      if (existsSync(examples)) {
        markdownFiles(
          examples,
          (_, example, description) => {
            fields(example, ["rows"]);
            const rows =
              example.rows === undefined ? null : list(example.rows, "rows");
            if (rows?.some((row) => !(row instanceof Map))) {
              throw new Error("Expected objects in example rows");
            }
            insert("example", {
              component: name,
              description,
              properties: rows === null ? null : exampleJson(rows),
            });
          },
          exampleHeader,
        );
      }
    });
  }

  try {
    sqlFile("schema.sql");
    collection("categories", "reference_category", [
      "id",
      "title",
      "sort_order",
    ]);
    collection("guides", "guide", [
      "title",
      "link",
      "description",
      "sort_order",
    ]);
    collection("projects", "example_cards", [
      "title",
      "folder",
      "db_engine",
      "description",
    ]);
    const sharedPath = join(content, "shared-parameters.yaml");
    const shared = at(sharedPath, () => {
      const entries = parse(read(sharedPath));
      for (const field of Object.values(entries)) parameter(field);
      return entries;
    });
    references("components", shared);
    references("functions", shared);
    markdownFiles(join(content, "blog"), (_, meta, body) => {
      fields(meta, ["title", "date", "icon", "summary", "external_url"]);
      insert("blog_posts", {
        title: text(meta.title, "title"),
        created_at: text(meta.date, "date"),
        icon: text(meta.icon, "icon"),
        description: text(meta.summary, "summary"),
        external_url: meta.external_url,
        content: body || null,
      });
    });
    for (const [name, option] of Object.entries(schema.properties)) {
      at(`Configuration option ${name}`, () =>
        insert("config_option", {
          name,
          type: schemaType(option),
          default_md: configDefault(option),
          description_md: text(option.description, "description"),
        }),
      );
    }
    sqlFile("search.sql");
    return `${statements.join("\n")}\n`;
  } finally {
    db.close();
  }
}
