import { expect, type Page, test } from "@playwright/test";

const BASE = process.env.SQLPAGE_URL || "http://localhost:8080";
const TEST_USER = { username: "demo", password: "demo" };

test.describe("OIDC SSO Authentication", () => {
  test.beforeEach(async ({ context }) => {
    await context.clearCookies();
  });

  test("public page accessible without authentication", async ({ page }) => {
    await page.goto(BASE);
    await expect(page.getByRole("heading", { name: "Welcome" })).toBeVisible();
    await expect(page.getByText("browsing as a guest")).toBeVisible();
    await expect(page.getByRole("link", { name: "Log in" })).toBeVisible();
  });

  test("protected page redirects to OIDC provider", async ({ page }) => {
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/protected") && response.status() === 303,
    );
    await page.goto(`${BASE}/protected`);
    const response = await responsePromise;
    expect(response.status()).toBe(303);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);
    await expect(page.locator("#username")).toBeVisible();
  });

  test("full login flow with valid credentials", async ({ page }) => {
    await page.goto(`${BASE}/protected`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);

    await page.locator("#username").fill(TEST_USER.username);
    await page.locator("#password").fill(TEST_USER.password);
    await page.locator("#kc-login").click();

    await page.waitForURL(`${BASE}/protected`);
    await expect(
      page.getByRole("heading", { name: /You're in, Demo User/ }),
    ).toBeVisible();
    await expect(page.getByText("demo@example.com")).toBeVisible();
  });

  test("user info functions return correct claims", async ({ page }) => {
    await loginWithKeycloak(page);
    await page.goto(`${BASE}/protected`);
    await page.waitForURL(`${BASE}/protected`);

    await expect(page.getByText("demo@example.com")).toBeVisible();
    await expect(page.getByTitle("sub")).toBeVisible();
    await expect(page.getByTitle("email")).toBeVisible();
    await expect(page.getByTitle("name")).toBeVisible();
  });

  test("logout clears authentication and removes cookie", async ({
    page,
    context,
  }) => {
    await loginWithKeycloak(page);
    await page.goto(`${BASE}/logout`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);

    const cookies = await context.cookies();
    const authCookie = cookies.find((c) => c.name === "sqlpage_auth");
    expect(authCookie).toBeUndefined();
    await page.goto(BASE);
    await expect(page.getByText("browsing as a guest")).toBeVisible();
  });

  test("authenticated user sees personalized home page", async ({ page }) => {
    await loginWithKeycloak(page);
    await page.goto(BASE);

    await expect(
      page.getByRole("heading", { name: /Welcome back, Demo User/ }),
    ).toBeVisible();
    await expect(page.getByText("demo@example.com")).toBeVisible();
    await expect(page.getByRole("link", { name: "log out" })).toBeVisible();
  });

  test("protected public path is accessible without auth", async ({ page }) => {
    await page.goto(`${BASE}/protected/public/hello.jpeg`);
    const response = await page.waitForResponse((r) =>
      r.url().includes("/protected/public/hello.jpeg"),
    );
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("image/jpeg");
  });

  test("invalid auth cookie is handled gracefully", async ({
    page,
    context,
  }) => {
    await context.addCookies([
      {
        name: "sqlpage_auth",
        value: "invalid.jwt.token",
        domain: "localhost",
        path: "/",
      },
    ]);
    await page.goto(`${BASE}/protected`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);
    await expect(page.locator("#username")).toBeVisible();
  });

  test("expired token triggers re-authentication", async ({
    page,
    context,
  }) => {
    const expiredJwt =
      "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwiIiA6ICJQIn0." +
      "eyJleHAiOjEsImlhdCI6MSwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdDo4MTgxL3JlYWxtcy9zcWxwYWdlX2RlbW8iLCJhdWQiOiJzcWxwYWdlIiwic3ViIjoiMTIzIn0." +
      "signature";
    await context.addCookies([
      {
        name: "sqlpage_auth",
        value: expiredJwt,
        domain: "localhost",
        path: "/",
      },
    ]);
    await page.goto(`${BASE}/protected`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);
    await expect(page.locator("#username")).toBeVisible();
  });

  test("login preserves original target URL", async ({ page }) => {
    await page.goto(`${BASE}/protected?foo=bar`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);

    await page.locator("#username").fill(TEST_USER.username);
    await page.locator("#password").fill(TEST_USER.password);
    await page.locator("#kc-login").click();

    await page.waitForURL(/.*\/protected\?foo=bar/);
  });

  test("failed login stays on Keycloak login page", async ({ page }) => {
    await page.goto(`${BASE}/protected`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);

    await page.locator("#username").fill("wrong");
    await page.locator("#password").fill("credentials");
    await page.locator("#kc-login").click();

    await expect(page.getByText(/Invalid username or password/)).toBeVisible();
    expect(page.url()).toContain(
      "/realms/sqlpage_demo/protocol/openid-connect",
    );
  });

  test("CSRF state cookie is set during login flow", async ({
    page,
    context,
  }) => {
    await page.goto(`${BASE}/protected`);
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);

    const cookies = await context.cookies();
    const stateCookie = cookies.find((c) =>
      c.name.startsWith("sqlpage_oidc_state_"),
    );
    expect(stateCookie).toBeDefined();
    expect(stateCookie?.httpOnly).toBe(true);
  });

  test("nonce cookie is set after successful login", async ({
    page,
    context,
  }) => {
    await loginWithKeycloak(page);

    const cookies = await context.cookies();
    const nonceCookie = cookies.find((c) => c.name === "sqlpage_oidc_nonce");
    expect(nonceCookie).toBeDefined();
    expect(nonceCookie?.httpOnly).toBe(true);
  });

  test("auth cookie has correct security attributes", async ({
    page,
    context,
  }) => {
    await loginWithKeycloak(page);

    const cookies = await context.cookies();
    const authCookie = cookies.find((c) => c.name === "sqlpage_auth");
    expect(authCookie).toBeDefined();
    expect(authCookie?.httpOnly).toBe(true);
    expect(authCookie?.sameSite).toBe("Lax");
    expect(authCookie?.path).toBe("/");
  });

  test("multiple protected pages work with single login", async ({ page }) => {
    await loginWithKeycloak(page);

    await page.goto(`${BASE}/protected`);
    await expect(page.getByText("demo@example.com")).toBeVisible();

    await page.goto(BASE);
    await expect(
      page.getByRole("heading", { name: /Welcome back/ }),
    ).toBeVisible();

    await page.goto(`${BASE}/protected`);
    await expect(page.getByText("demo@example.com")).toBeVisible();
  });

  test("callback endpoint returns error for missing state", async ({
    page,
  }) => {
    const response = await page.goto(`${BASE}/sqlpage/oidc_callback?code=test`);
    expect(response?.status()).toBeGreaterThanOrEqual(300);
  });

  test("callback endpoint with invalid code redirects to OIDC", async ({
    page,
    context,
  }) => {
    await context.addCookies([
      {
        name: "sqlpage_oidc_state_test_state",
        value: JSON.stringify({ n: "test_nonce", r: "/" }),
        domain: "localhost",
        path: "/",
      },
    ]);
    await page.goto(
      `${BASE}/sqlpage/oidc_callback?code=invalid&state=test_state`,
    );
    await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);
  });
});

async function loginWithKeycloak(page: Page) {
  await page.goto(`${BASE}/protected`);
  await page.waitForURL(/.*\/realms\/sqlpage_demo\/protocol\/openid-connect/);
  await page.locator("#username").fill(TEST_USER.username);
  await page.locator("#password").fill(TEST_USER.password);
  await page.locator("#kc-login").click();
  await page.waitForURL(`${BASE}/protected`);
}
