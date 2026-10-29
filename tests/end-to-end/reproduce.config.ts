import { defineConfig } from "@playwright/test";
import base from "./playwright.config.ts";

export default defineConfig({
  ...base,
  projects: base.projects?.filter((project) => project.name === "fixtures"),
  webServer: [base.webServer]
    .flat()
    .flatMap((server) => (server?.name === "fixtures" ? [server] : [])),
});
