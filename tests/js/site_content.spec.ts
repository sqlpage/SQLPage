import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import test from "node:test";
import { buildDocumentationSeed } from "../../scripts/site-content.mjs";

test("authored examples keep their order, quoting, and optional previews in the startup migration", (t) => {
  const content = mkdtempSync(join(tmpdir(), "sqlpage-content-"));
  t.after(() => rmSync(content, { recursive: true, force: true }));
  for (const dir of ["components", "functions", "blog"])
    mkdirSync(join(content, dir));
  for (const manifest of ["categories", "guides", "projects"]) {
    writeFileSync(join(content, `${manifest}.yaml`), "[]\n");
  }
  writeFileSync(
    join(content, "shared-parameters.yaml"),
    "id: {name: id, type: TEXT, description: HTML id}\n",
  );
  const topic = join(content, "components", "demo.md");
  writeFileSync(topic, "---\ntop: [id]\n---\n\nA contributor's demo.\n");
  const examples = join(content, "components", "demo", "examples");
  mkdirSync(examples, { recursive: true });
  writeFileSync(
    join(examples, "02-result.md"),
    `---
rows:
  - component: text
    contents: "Reader's choice"
    "2": 9007199254740993
    "1": 42
---
Result
`,
  );
  writeFileSync(join(examples, "01-notes.md"), "Just a note.\n");
  const schema = {
    properties: {
      toggle: { type: "boolean", default: false, description: "Turn this on." },
      recipient: { type: ["string", "null"], description: "Recipient." },
      count: {
        type: "integer",
        minimum: 0,
        maximum: 8,
        "x-default-description": "Depends on the database.",
        description: "Limit.",
      },
    },
  };
  const db = new DatabaseSync(":memory:");
  t.after(() => db.close());
  const seed = buildDocumentationSeed(schema, content);
  assert.equal(seed, buildDocumentationSeed(schema, content));
  db.exec(seed);
  assert.deepEqual(
    db
      .prepare(
        "SELECT description, source FROM example_with_source WHERE component = 'demo' ORDER BY example_number",
      )
      .all()
      .map((row) => [row.description, row.source]),
    [
      ["Just a note.", null],
      [
        "Result",
        `select\n    'text' as "component",\n    'Reader''s choice' as "contents",\n    9007199254740993 as "2",\n    42 as "1";`,
      ],
    ],
  );
  assert.deepEqual(
    db
      .prepare("SELECT name, type, default_md FROM config_option ORDER BY name")
      .all()
      .map((row) => [row.name, row.type, row.default_md]),
    [
      ["count", "integer (0 to 8)", "Depends on the database."],
      ["recipient", "string or null", null],
      ["toggle", "boolean", "`false`"],
    ],
  );
  assert.equal(
    db
      .prepare(
        "SELECT fragment FROM documentation_fts WHERE documentation_fts MATCH 'recipient' AND page = '/configuration'",
      )
      .get()?.fragment,
    "recipient",
  );

  writeFileSync(topic, "---\ntop: [missing]\n---\nExample.\n");
  assert.throws(
    () => buildDocumentationSeed(schema, content),
    /demo\.md: Unknown shared parameter "missing"/,
  );
  renameSync(topic, join(content, "components", "moved.md"));
  assert.throws(
    () => buildDocumentationSeed(schema, content),
    /no corresponding reference demo\.md/,
  );
});
