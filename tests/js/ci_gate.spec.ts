import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

type Job = { result: string; outputs?: Record<string, string> };
const root = fileURLToPath(new URL("../../", import.meta.url));
const coreJobs = [
  "compile_and_lint",
  "published_crate",
  "fuzz_sql_regression_fixtures",
  "test",
  "windows_test",
  "playwright",
  "docker_build",
  "hurl_examples",
];

function results(full = true, examples: string[] = []) {
  const jobs: Record<string, Job> = Object.fromEntries(
    coreJobs.map((name) => [name, { result: full ? "success" : "skipped" }]),
  );
  jobs.ci_selection = { result: "success", outputs: { full: String(full) } };
  if (full)
    jobs.hurl_examples = {
      result: "success",
      outputs: { examples: JSON.stringify(examples) },
    };
  jobs.hurl = { result: full && examples.length > 0 ? "success" : "skipped" };
  return jobs;
}

function check(jobs: unknown) {
  const run = spawnSync(process.execPath, ["scripts/check-ci-results.mjs"], {
    cwd: root,
    env: { ...process.env, NEEDS_JSON: JSON.stringify(jobs) },
    encoding: "utf8",
    timeout: 5_000,
  });
  assert.ifError(run.error);
  assert.equal(run.signal, null, run.stderr);
  return run.status;
}

test("CI gate accepts intentionally skipped suites for a docs-only PR", () => {
  assert.equal(check(results(false)), 0);
});

test("CI gate accepts successful suites and an empty Hurl selection", () => {
  assert.equal(check(results()), 0);
});

test("CI gate accepts the selected Hurl examples after they pass", () => {
  assert.equal(check(results(true, ["examples/single sign on"])), 0);
});

for (const name of [...coreJobs, "ci_selection", "hurl"]) {
  for (const result of ["failure", "cancelled"]) {
    test(`CI gate blocks ${name} ${result}`, () => {
      const jobs = results(true, ["examples/single sign on"]);
      jobs[name] = { ...jobs[name], result };
      assert.notEqual(check(jobs), 0);
    });
  }
}

for (const name of coreJobs) {
  test(`CI gate blocks an unexpected ${name} skip`, () => {
    const jobs = results();
    jobs[name] = { ...jobs[name], result: "skipped" };
    assert.notEqual(check(jobs), 0);
  });
}

test("CI gate blocks skipped Hurl tests when examples were selected", () => {
  const jobs = results(true, ["examples/single sign on"]);
  jobs.hurl = { result: "skipped" };
  assert.notEqual(check(jobs), 0);
});

test("CI gate blocks missing results from a required job", () => {
  const jobs = results();
  delete jobs.test;
  assert.notEqual(check(jobs), 0);
});

for (const full of [undefined, "", "unknown"]) {
  test(`CI gate blocks an ambiguous selection: ${String(full)}`, () => {
    const jobs = results(false);
    jobs.ci_selection = {
      result: "success",
      outputs: full === undefined ? {} : { full },
    };
    assert.notEqual(check(jobs), 0);
  });
}

test("CI gate blocks a skipped change detector", () => {
  const jobs = results(false);
  jobs.ci_selection = { result: "skipped" };
  assert.notEqual(check(jobs), 0);
});

for (const examples of ["", "invalid-json", "null", "{}", '[""]', "[true]"]) {
  test(`CI gate blocks an invalid Hurl selection: ${examples}`, () => {
    const jobs = results();
    jobs.hurl_examples = { result: "success", outputs: { examples } };
    assert.notEqual(check(jobs), 0);
  });
}

test("CI gate rejects failed jobs even on a docs-only selection", () => {
  const jobs = results(false);
  jobs.test = { result: "failure" };
  assert.notEqual(check(jobs), 0);
});

for (const jobs of [null, {}, [], { ci_selection: { result: "success" } }]) {
  test(`CI gate blocks malformed or incomplete result data: ${JSON.stringify(jobs)}`, () => {
    assert.notEqual(check(jobs), 0);
  });
}
