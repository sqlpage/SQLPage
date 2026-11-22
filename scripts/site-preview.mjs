import { execFileSync, spawn } from "node:child_process";
import { buildSite, repository, siteDirectory } from "./site-build.mjs";

buildSite();

// Ask Cargo for the actual executable so a custom target directory also works.
// Running the server directly lets Node's watcher stop it before restarting.
const output = execFileSync(
  "cargo",
  [
    "build",
    "--locked",
    "--bin",
    "sqlpage",
    "--message-format=json-render-diagnostics",
  ],
  {
    cwd: repository,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    maxBuffer: 16 * 1024 * 1024,
  },
);
const binary = output
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line))
  .find(
    (message) => message.executable && message.target.name === "sqlpage",
  ).executable;
const server = spawn(binary, [], { cwd: siteDirectory, stdio: "inherit" });
for (const signal of ["SIGTERM", "SIGINT"]) {
  process.once(signal, () => server.kill(signal));
}
server.on("exit", (status, signal) => {
  process.exitCode =
    status ?? (signal === "SIGTERM" || signal === "SIGINT" ? 0 : 1);
});
