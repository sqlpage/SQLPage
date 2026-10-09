# Working on SQLPage

## Keep guidance small

This file records durable architectural constraints and instructions for agents.
[CONTRIBUTING.md](./CONTRIBUTING.md) covers contributor setup and validation;
[CI](./.github/workflows/ci.yml) defines the supported test matrix. Discover module
layout, APIs, schemas, and detailed behavior from the code and nearby tests.

Edit and prune these documents rather than appending after every task. Add guidance
only when it expresses a lasting rule that cannot be made clear in code. Put local
implementation caveats beside the implementation or regression test. Keep user-facing
reference material in the official documentation site. Do not duplicate it here.

## Architectural contracts

- SQL files execute statements sequentially and stream component rows in response
  order. Preserve streaming, cancellation, transaction rollback, and contextual
  errors; do not hide failures through unrelated error-handling changes.
- Request inputs are bound parameters, never interpolated into SQL. Preserve the
  distinction between request inputs and mutable SET variables, including explicit
  NULL and child execution isolation.
- Database behavior must remain portable across supported drivers. Isolate
  engine-specific SQL and verify execution, binding, and decoding changes against
  the affected CI matrix entries.
- Nested SQL execution must release an active fetch stream before reusing its
  connection. Preserve one-connection regression coverage, output column ordering,
  and exclusion of private function inputs from responses.
- Extend existing module and fixture patterns before introducing abstractions.
  Browser component tests must exercise SQL fixtures and normal page initialization
  through the shared Playwright harness.

## Changes and validation

Keep changes scoped to the task, including formatting. Reuse existing tests and
parameterize repeated setup without removing assertions or scenarios. Run the
relevant tests and the formatting/lint checks in CONTRIBUTING.md before stopping.
Report unavailable checks and failures accurately; a test summary followed by a
hung process is a failure.

The official site in `examples/official-site/` is rebuilt from its SQL migrations
on deployment. Update the existing page or documentation migration for changed
behavior; add a migration only for new documentation content. Document user-visible
changes there and in configuration.md when applicable. Add a concise CHANGELOG.md
entry only for user-visible changes, and never edit an already tagged release.
