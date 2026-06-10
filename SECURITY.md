# Security Policy

## Reporting a Vulnerability

Please report suspected SQLPage vulnerabilities privately to
`contact@ophir.dev`.

Include the SQLPage version or commit, database backend, relevant
configuration, a minimal SQL file if possible, and the exact attacker-controlled
input. Do not open a public issue for a non-public vulnerability.

## Threat Model

SQLPage is a runtime for applications written in SQL. It maps HTTP requests to
SQL files, executes those files on the configured database, and renders the
result. SQLPage is not a sandbox for SQLPage application authors or operators.

### Trusted Inputs

SQLPage trusts the application and deployment:

- application files under `web_root`;
- application files stored in the optional `sqlpage_files` table;
- configuration, command-line arguments, environment variables, and `.env`
  files;
- templates, migrations, and connection-management SQL in the configuration
  directory;
- database behavior and access: roles, permissions, schema, extensions,
  triggers, stored procedures, views, and migrations;
- anyone or anything that can modify one of the above.

Control of trusted inputs is control of the SQLPage application. If an attacker
can edit `sqlpage.json`, alter templates, change OIDC path rules, enable
dangerous Markdown options, enable `allow_exec`, or write SQL into
`sqlpage_files`, that is outside SQLPage's vulnerability boundary unless
SQLPage itself granted that access to an untrusted actor.

### Untrusted Inputs

SQLPage does not trust remote inputs:

- HTTP paths, query strings, form fields, request bodies, and multipart uploads;
- uploaded filenames, MIME types, and file contents;
- HTTP headers, cookies, Basic Auth credentials, and unauthenticated OIDC
  callback parameters;
- responses from remote servers contacted with `sqlpage.fetch` or
  `sqlpage.fetch_with_meta`.

Database row values are not classified globally as trusted or untrusted.
SQLPage cannot know whether a row came from an administrator-maintained table,
user-generated content, a trigger, or a third-party import. The security
boundary depends on where trusted SQL places the value.

### Data Positions and Instruction Positions

Trusted SQL chooses whether a value is ordinary data or an instruction to
SQLPage.

Data positions are values SQLPage should render, serialize, encode, or pass as
bound database parameters without giving them extra authority. Examples include
ordinary table cells, text fields, JSON response data, CSV cells, and safe
Markdown.

Instruction positions are values SQLPage intentionally treats as application
instructions or capability arguments. Examples include:

- component names;
- `dynamic` component properties;
- response status codes, headers, redirects, cookies, and downloads;
- file paths passed to `run_sql`, `read_file_as_text`, or
  `read_file_as_data_url`;
- URLs and request options passed to `fetch` or `fetch_with_meta`;
- raw HTML and unsafe Markdown;
- command names and arguments passed to `exec` when `allow_exec` is enabled.

Placing a database value in an instruction position is an application decision.
For example, `select sqlpage.read_file_as_text(f) from trusted_table` is allowed
by design. If someone who can modify `trusted_table` can read arbitrary files,
that is a problem in the application or database permissions, not SQLPage.

SQLPage's responsibility is to enforce the documented meaning and guardrails of
each position: data positions must not become instructions, instruction
positions must not bypass their documented checks, and malformed values must not
crash SQLPage or escape into unrelated capabilities.

## In Scope

Please report cases where SQLPage itself crosses that boundary. Examples:

- An HTTP request can execute attacker-chosen SQL without trusted SQL explicitly
  exposing that behavior.
- SQLPage parameter handling turns `$name`, `:name`, or `?name` into executable
  SQL instead of a bound database value.
- A value in a data position causes SQL execution, command execution, host file
  access, response-header injection, unsafe HTML execution, or another
  instruction-position effect.
- A database value in any position reliably crashes or panics SQLPage instead
  of producing a response or application-level error.
- HTTP routing, path decoding, static file serving, caching, `run_sql`, or file
  functions expose host files that trusted SQL or configuration did not select.
- Reserved private files, including the `sqlpage/` prefix, dotfiles,
  templates, and configuration, are reachable over HTTP.
- `allow_exec` is false, but an attacker can execute a local command through
  SQLPage.
- Built-in OIDC handling accepts forged, expired, wrong-issuer,
  wrong-audience, wrong-nonce, or wrong-signature tokens, or applies configured
  public/protected path rules incorrectly.
- Default-safe rendering or safe Markdown executes browser script.
- SQLPage-generated production error responses expose source code, stack
  traces, SQL text, parameters, environment values, or configuration values.
- Upload handling allows path traversal, overwrite of unintended files, or file
  disclosure without trusted SQL selecting that behavior.
- Official SQLPage documentation or examples recommend placing untrusted remote
  input into an instruction position without validation.

## Out of Scope

The following are usually application or deployment vulnerabilities, not
SQLPage vulnerabilities:

- Trusted SQL omits authentication or authorization checks.
- Trusted SQL, a stored procedure, trigger, view, or extension builds and
  executes SQL from untrusted data.
- Trusted SQL selects a value into an instruction position such as `component`,
  `dynamic.properties`, a redirect target, header value, file path, `run_sql`
  target, `fetch` URL, raw HTML, unsafe Markdown, or `exec` argument.
- Trusted SQL intentionally reads a host file, including an absolute path, and
  returns it to a client.
- An operator intentionally changes configuration to expose files, trust a
  different database, make an OIDC path public, weaken CSP, enable dangerous
  Markdown options, load SQLite extensions, or enable `allow_exec`.
- An attacker can modify SQL files, templates, configuration, environment
  variables, migrations, database code, or `sqlpage_files`.
- The configured database role has broader permissions than the application
  needs.
- A SQLPage application is publicly reachable because no authentication was
  configured.
- An attacker can plant or overwrite cookies for the SQLPage origin (for
  example through a compromised subdomain, a sibling application on a shared
  parent domain, or a man-in-the-middle on plain HTTP). Attacks that depend on
  injecting attacker-chosen cookies into the victim's browser, such as OIDC
  login CSRF or session fixation via a forged login-flow-state cookie, are out
  of scope. SQLPage assumes its origin's cookie jar is writable only by the
  user agent, not by attackers.
- Trusted SQL asks SQLPage or the database to perform expensive work.

These may still be serious and should be fixed in the affected application,
deployment, or documentation.

## Boundary Examples

Report: `/x.sql?sort=...` causes SQLPage to execute attacker-chosen SQL because
SQLPage rewrote a parameter incorrectly.

Do not report as SQLPage: `x.sql` passes `sort` to a stored procedure that
concatenates it into dynamic SQL.

Report: a normal table cell containing `<script>` executes script in the
browser when rendered by a default-safe component.

Do not report as SQLPage: trusted SQL selects that value as `html`,
`unsafe_contents_md`, `component`, or `dynamic.properties`.

Report: a specific string returned by the database in a text column panics the
SQLPage process.

Do not report as SQLPage: trusted SQL passes that string as the filename
argument to `sqlpage.read_file_as_text`.

Report: `/..%2F..%2Fetc%2Fpasswd` or `/sqlpage/sqlpage.json` is served directly
by SQLPage.

Do not report as SQLPage: trusted SQL calls
`sqlpage.read_file_as_text('/etc/passwd')` and renders the result.

Report: an unauthenticated request can write a new SQL file into
`sqlpage_files` through an unintended SQLPage endpoint.

Do not report as SQLPage: an administrator, migration, or intentionally exposed
application page writes SQL into `sqlpage_files`.

Report: official documentation recommends `sqlpage.run_sql($user_input)`.

Do not report as SQLPage: a private application uses
`sqlpage.run_sql($user_input)` despite the documentation warning against it.
