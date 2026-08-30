import { defineConfig, devices } from "@playwright/test";

const fixtureBaseURL =
  process.env.SQLPAGE_FIXTURE_BASE ?? "http://127.0.0.1:8081";

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
      use: {
        ...devices["Desktop Chrome"],
        baseURL: process.env.SQLPAGE_TEST_BASE ?? "http://127.0.0.1:8080",
      },
    },
    {
      name: "fixtures",
      testMatch: "fixtures/**/test.ts",
      use: { ...devices["Desktop Chrome"], baseURL: fixtureBaseURL },
    },
  ],
  webServer: process.env.CI
    ? undefined
    : {
        command:
          "cargo run --manifest-path ../../Cargo.toml -- --web-root fixtures --config-dir fixture-server",
        url: fixtureBaseURL,
        reuseExistingServer: true,
        timeout: 120_000,
      },
});
