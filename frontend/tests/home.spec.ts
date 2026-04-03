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
  // Scope all task locators to desktop layout to avoid duplicate elements
  // (mobile layout exists in DOM but hidden via CSS)
  const board = ".desktop-layout";

  test.beforeEach(async ({ page, rpc }) => {
    const email = `${uniqueId("task")}@example.com`;

    await rpc("register", {
      name: "Task User",
      email,
      password: "password123",
    });

    await page.goto("/login");
    await page.getByPlaceholder("Email").fill(email);
    await page.getByPlaceholder("Password").fill("password123");
    await page.getByRole("button", { name: "Sign in" }).click();
    await expect(page.locator(".dashboard")).toBeVisible({ timeout: 15000 });
  });

  test("create a task via column add button", async ({ page }) => {
    const column = page.locator(`${board} .board-column`).first();
    await column.locator(".column-add-btn").click();
    await page.locator(".add-task-input").fill("My first task");
    await page.locator(".add-task-submit").click();

    await expect(
      page.locator(`${board} .task-card-title`, { hasText: "My first task" }),
    ).toBeVisible({ timeout: 10000 });
  });

  test("quick capture via n key", async ({ page }) => {
    await page.locator(".dashboard").press("n");
    await expect(page.locator(".capture-input")).toBeVisible();
    await expect(page.locator(".capture-input")).toBeFocused();

    await page.locator(".capture-input").fill("Keyboard task");
    await page.locator(".capture-input").press("Enter");

    await expect(page.locator(".capture-input")).not.toBeVisible();
    await expect(
      page.locator(`${board} .task-card-title`, { hasText: "Keyboard task" }),
    ).toBeVisible({ timeout: 10000 });
  });

  test("edit task title in detail panel", async ({ page }) => {
    const column = page.locator(`${board} .board-column`).first();
    await column.locator(".column-add-btn").click();
    await page.locator(".add-task-input").fill("Edit me");
    await page.locator(".add-task-submit").click();
    await expect(
      page.locator(`${board} .task-card-title`, { hasText: "Edit me" }),
    ).toBeVisible({ timeout: 10000 });

    // Click title to open detail panel
    await page
      .locator(`${board} .task-card-title`, { hasText: "Edit me" })
      .click();
    await expect(page.locator(".detail-panel")).toBeVisible();

    // Edit the title
    const titleInput = page.locator(".detail-title-input");
    await titleInput.clear();
    await titleInput.fill("Edited title");
    await titleInput.blur();

    // Close and verify
    await page.locator(".detail-close").click();
    await expect(
      page.locator(`${board} .task-card-title`, { hasText: "Edited title" }),
    ).toBeVisible({ timeout: 10000 });
  });

  test("delete a task", async ({ page }) => {
    const column = page.locator(`${board} .board-column`).first();
    await column.locator(".column-add-btn").click();
    await page.locator(".add-task-input").fill("To delete");
    await page.locator(".add-task-submit").click();

    await expect(
      page.locator(`${board} .task-card-title`, { hasText: "To delete" }),
    ).toBeVisible({ timeout: 10000 });

    // Hover and delete
    const card = page.locator(`${board} .task-card`, { hasText: "To delete" });
    await card.hover();
    await card.locator(".task-action-delete").click();

    // Undo toast should appear
    await expect(page.locator(".undo-toast")).toBeVisible({ timeout: 5000 });

    // Task disappears after 5s undo window + server processing
    await expect(
      page.locator(`${board} .task-card-title`, { hasText: "To delete" }),
    ).not.toBeVisible({ timeout: 15000 });
  });
});
