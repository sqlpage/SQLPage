// This always-run gate accepts only skips justified by successful selection.
function fail(message) {
  console.error(message);
  process.exit(1);
}

let jobs;
try {
  jobs = JSON.parse(process.env.NEEDS_JSON ?? "");
} catch {
  fail("Missing or invalid CI job results.");
}
if (!jobs || typeof jobs !== "object" || Array.isArray(jobs))
  fail("CI job results must be an object.");

if (jobs.ci_selection?.result !== "success")
  fail("CI change detection did not succeed.");
const full = jobs.ci_selection.outputs?.full;
if (full !== "true" && full !== "false")
  fail("CI change detection returned an ambiguous selection.");

for (const [name, job] of Object.entries(jobs)) {
  if (!["success", "skipped"].includes(job?.result))
    fail(`${name} did not pass: ${job?.result ?? "missing result"}.`);
}

if (full === "true") {
  for (const name of [
    "compile_and_lint",
    "published_crate",
    "fuzz_smoke",
    "test",
    "windows_test",
    "playwright",
    "docker_build",
    "hurl_examples",
  ]) {
    if (jobs[name]?.result !== "success")
      fail(`Required job ${name} did not run successfully.`);
  }

  let examples;
  try {
    examples = JSON.parse(jobs.hurl_examples.outputs?.examples ?? "");
  } catch {
    fail("Hurl selection returned invalid JSON.");
  }
  if (
    !Array.isArray(examples) ||
    !examples.every((item) => typeof item === "string" && item.length > 0)
  )
    fail("Hurl selection must be an array of example names.");
  const allowed = examples.length > 0 ? ["success"] : ["success", "skipped"];
  if (!allowed.includes(jobs.hurl?.result))
    fail("Selected Hurl tests did not run successfully.");
}

console.log("All selected CI jobs passed.");
