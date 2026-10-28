import path from "node:path";
import { defineConfig, devices } from "@playwright/test";

const repository = path.resolve(import.meta.dirname, "../..");

// The release images start SQLPage from their CMD, so the command names the
// binary again after the image.
function releaseImage(release: string) {
  return [
    "docker run --rm --init --network host --env SQLPAGE_LISTEN_ON --env SQLPAGE_DATABASE_URL",
    `--volume "${repository}:${repository}" --workdir "$PWD"`,
    `docker.io/lovasoa/sqlpage:${release} /usr/local/bin/sqlpage`,
  ].join(" ");
}

const sqlpage = process.env.SQLPAGE_RELEASE
  ? releaseImage(process.env.SQLPAGE_RELEASE)
  : (process.env.SQLPAGE_BINARY ??
    "cargo run --manifest-path ../../Cargo.toml --");

const anyFreePort = "127.0.0.1:0";
const compileAndStart = 600_000;
const stopTheContainerToo = { signal: "SIGTERM", timeout: 5_000 } as const;

const fixturesDatabase: Record<string, string> = process.env
  .FIXTURES_DATABASE_URL
  ? { SQLPAGE_DATABASE_URL: process.env.FIXTURES_DATABASE_URL }
  : {};

// Releases before v0.47.0 announce the port they were given rather than the one
// they bound, so a release listens on a fixed port it can announce correctly.
function listenOn(releasePort: number) {
  return process.env.SQLPAGE_RELEASE ? `127.0.0.1:${releasePort}` : anyFreePort;
}

// Playwright uppercases the named capture group of `wait` into the environment
// it hands the worker processes, which is where the tests read the address back
// from: https://playwright.dev/docs/api/class-testconfig#test-config-web-server
function announcedAddress(variable: string) {
  return {
    announcement: new RegExp(
      `(?:View your website at:.*?http://|Now listening on )(?<${variable.toLowerCase()}>\\S+)`,
      "s",
    ),
    url: `http://${process.env[variable]}`,
  };
}

const officialSite = announcedAddress("SQLPAGE_OFFICIAL_SITE_ADDRESS");
const fixtures = announcedAddress("SQLPAGE_FIXTURES_ADDRESS");

export default defineConfig({
  testDir: ".",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  // Keep live progress when CI times out, alongside the downloadable HTML report.
  reporter: process.env.CI ? [["line"], ["html"]] : "html",
  use: {
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "official-site",
      testMatch: "*.spec.ts",
      use: { ...devices["Desktop Chrome"], baseURL: officialSite.url },
    },
    {
      name: "fixtures",
      testMatch: "fixtures/**/test.ts",
      use: { ...devices["Desktop Chrome"], baseURL: fixtures.url },
    },
  ],
  webServer: [
    {
      name: "official site",
      command: sqlpage,
      cwd: "../../examples/official-site",
      env: { SQLPAGE_LISTEN_ON: listenOn(18080) },
      wait: { stderr: officialSite.announcement },
      timeout: compileAndStart,
      gracefulShutdown: stopTheContainerToo,
    },
    {
      name: "fixtures",
      command: `${sqlpage} --web-root fixtures --config-dir fixture-server`,
      env: { SQLPAGE_LISTEN_ON: listenOn(18081), ...fixturesDatabase },
      wait: { stderr: fixtures.announcement },
      timeout: compileAndStart,
      gracefulShutdown: stopTheContainerToo,
    },
  ],
});
