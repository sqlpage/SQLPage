import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

import { buildDocumentationSeed } from "./site-content.mjs";

const repository = resolve(import.meta.dirname, "..");
const siteDirectory = join(repository, "target", "official-site");
const source = join(repository, "examples", "official-site");

const excluded = new Set([
  "content",
  "README.md",
  "docker-compose.yml",
  "test.hurl",
  join("sqlpage", "sqlpage.db"),
  join("sqlpage", "https"),
]);

function buildSite() {
  const parent = dirname(siteDirectory);
  mkdirSync(parent, { recursive: true });
  const temporary = mkdtempSync(join(parent, ".official-site-"));
  const next = join(temporary, "next");
  const previous = join(temporary, "previous");
  try {
    cpSync(source, next, {
      recursive: true,
      filter: (file) => !excluded.has(relative(source, file)),
    });
    writeFileSync(
      join(next, "sqlpage", "migrations", "01_documentation.sql"),
      buildDocumentationSeed(),
    );
    copyFileSync(
      join(repository, "configuration.md"),
      join(next, "configuration.md"),
    );
    // Stage the complete site before exchanging the previous preview.
    try {
      renameSync(siteDirectory, previous);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
    try {
      renameSync(next, siteDirectory);
    } catch (error) {
      if (!existsSync(siteDirectory) && existsSync(previous)) {
        renameSync(previous, siteDirectory);
      }
      throw error;
    }
    console.log(`Prepared ${siteDirectory}`);
  } finally {
    rmSync(temporary, { force: true, recursive: true });
  }
}

buildSite();
