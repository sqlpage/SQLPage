import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import test from "node:test";
import { buildConfigurationMigration } from "../../scripts/site-configuration.mjs";

test("the configuration migration keeps types, defaults, and per-setting search links", (t) => {
  const db = new DatabaseSync(":memory:");
  t.after(() => db.close());
  db.exec(`CREATE VIRTUAL TABLE documentation_fts USING fts5(
    title, description, example_source,
    page UNINDEXED, parameters UNINDEXED, fragment UNINDEXED
  );`);
  const schema = {
    properties: {
      toggle: { type: "boolean", default: false, description: "Turn this on." },
      recipient: {
        type: ["string", "null"],
        description: "Sender's recipient.",
      },
      count: {
        type: "integer",
        minimum: 0,
        maximum: 8,
        "x-default-description": "Depends on the database.",
        description: "Limit.",
      },
    },
  };
  db.exec(buildConfigurationMigration(schema));

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
        "SELECT description FROM documentation_fts WHERE documentation_fts MATCH 'recipient' AND page = '/configuration' AND fragment = 'recipient'",
      )
      .get()?.description,
    "Sender's recipient.",
  );
});
