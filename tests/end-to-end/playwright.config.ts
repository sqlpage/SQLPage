import { defineConfig, devices } from "@playwright/test";

const sqlpage =
  process.env.SQLPAGE_BINARY ?? "cargo run --manifest-path ../../Cargo.toml --";

const anyFreePort = "127.0.0.1:0";
const compileAndStart = 600_000;

// Playwright uppercases the named capture group of `wait` into the environment
// it hands the worker processes, which is where the tests read the address back
// from: https://playwright.dev/docs/api/class-testconfig#test-config-web-server
function announcedAddress(variable: string) {
  return {
    announcement: new RegExp(
      `View your website at:.*?http://(?<${variable.toLowerCase()}>\\S+)`,
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
  reporter: "html",
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
      env: { SQLPAGE_LISTEN_ON: anyFreePort },
      wait: { stderr: officialSite.announcement },
      timeout: compileAndStart,
    },
    {
      name: "fixtures",
      command: `${sqlpage} --web-root fixtures --config-dir fixture-server`,
      env: { SQLPAGE_LISTEN_ON: anyFreePort },
      wait: { stderr: fixtures.announcement },
      timeout: compileAndStart,
    },
  ],
});
