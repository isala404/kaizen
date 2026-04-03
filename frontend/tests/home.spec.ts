import {
  test,
  expect,
  trackConsoleErrors,
  uniqueId,
  API_URL,
} from "./fixtures";

test.describe("Auth", () => {
  test("unauthenticated user sees login page", async ({ page }) => {
    const errors = trackConsoleErrors(page);
    await page.goto("/");

    await expect(page.getByRole("heading", { name: "kaizen" })).toBeVisible();
    await expect(page.getByPlaceholder("Email")).toBeVisible();
    await expect(page.getByPlaceholder("Password")).toBeVisible();
  });

  test("register and land on dashboard", async ({ page, rpc }) => {
    const errors = trackConsoleErrors(page);
    const email = `${uniqueId("test")}@example.com`;

    await page.goto("/login");
    await expect(page.getByRole("heading", { name: "kaizen" })).toBeVisible();

    // Switch to register mode
    await page.getByRole("button", { name: /Register/ }).click();

    // Fill register form
    await page.getByPlaceholder("Name").fill("Test User");
    await page.getByPlaceholder("Email").fill(email);
    await page.getByPlaceholder("Password").fill("password123");
    await page.getByRole("button", { name: "Create account" }).click();

    // Should land on dashboard
    await expect(page.locator(".dashboard")).toBeVisible({ timeout: 15000 });
  });

  test("login with existing account", async ({ page, rpc }) => {
    const email = `${uniqueId("login")}@example.com`;

    // Register via RPC first
    await rpc("register", {
      name: "Login User",
      email,
      password: "password123",
    });

    await page.goto("/login");
    await page.getByPlaceholder("Email").fill(email);
    await page.getByPlaceholder("Password").fill("password123");
    await page.getByRole("button", { name: "Sign in" }).click();

    await expect(page.locator(".dashboard")).toBeVisible({ timeout: 15000 });
  });
});

test.describe("Tasks", () => {
  let email: string;
  let authToken: string;

  test.beforeEach(async ({ page, rpc }) => {
    email = `${uniqueId("task")}@example.com`;

    // Register and get token
    const resp = (await rpc("register", {
      name: "Task User",
      email,
      password: "password123",
    })) as { access_token: string; refresh_token: string };

    authToken = resp.access_token;

    // Login via UI
    await page.goto("/login");
    await page.getByPlaceholder("Email").fill(email);
    await page.getByPlaceholder("Password").fill("password123");
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page.locator(".dashboard")).toBeVisible({ timeout: 15000 });
  });

  test("create a task via column add button", async ({ page }) => {
    // Click the + button in the Inbox column
    await page
      .locator(".board-column")
      .first()
      .locator(".column-add-btn")
      .click();
    await page.locator(".add-task-input").fill("My first task");
    await page.locator(".add-task-submit").click();

    // Task should appear
    await expect(
      page.locator(".task-card-title", { hasText: "My first task" }),
    ).toBeVisible({ timeout: 10000 });
  });

  test("delete a task", async ({ page }) => {
    // Create a task first
    await page
      .locator(".board-column")
      .first()
      .locator(".column-add-btn")
      .click();
    await page.locator(".add-task-input").fill("To delete");
    await page.locator(".add-task-submit").click();

    await expect(
      page.locator(".task-card-title", { hasText: "To delete" }),
    ).toBeVisible({ timeout: 10000 });

    // Hover and delete
    const card = page.locator(".task-card", { hasText: "To delete" });
    await card.hover();
    await card.locator(".task-action-delete").click();

    // Task should disappear
    await expect(
      page.locator(".task-card-title", { hasText: "To delete" }),
    ).not.toBeVisible({ timeout: 10000 });
  });
});
