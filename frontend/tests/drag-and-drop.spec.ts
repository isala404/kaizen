import { test, expect, uniqueId, API_URL } from "./fixtures";
import { type Page, type Locator } from "@playwright/test";

const board = ".desktop-layout";

const COL = { inbox: 1, upNext: 2, paused: 3, done: 4 } as const;

function column(page: Page, idx: number): Locator {
  return page.locator(
    `${board} .board-columns > .board-column:nth-child(${idx})`,
  );
}

function cards(page: Page, colIdx: number): Locator {
  return column(page, colIdx).locator(".task-card");
}

function cardByTitle(page: Page, title: string): Locator {
  return page.locator(`${board} .task-card`, { hasText: title });
}

async function createTasksViaApi(
  token: string,
  tasks: { title: string; status: string }[],
) {
  for (const task of tasks) {
    const res = await fetch(`${API_URL}/_api/rpc/create_task`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        args: { title: task.title, status: task.status },
      }),
    });
    if (!res.ok) {
      throw new Error(`create_task failed: ${res.status} ${await res.text()}`);
    }
  }
}

async function loginAndWait(page: Page, email: string, password: string) {
  await page.goto("/login");
  await page.getByPlaceholder("Email").fill(email);
  await page.getByPlaceholder("Password").fill(password);
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.locator(".dashboard")).toBeVisible({ timeout: 15_000 });
}

async function dragCard(page: Page, source: Locator, target: Locator) {
  await source.waitFor({ state: "visible" });
  await target.waitFor({ state: "visible" });

  const srcBox = (await source.boundingBox())!;
  const tgtBox = (await target.boundingBox())!;

  const srcCenter = {
    x: srcBox.x + srcBox.width / 2,
    y: srcBox.y + srcBox.height / 2,
  };
  const tgtCenter = {
    x: tgtBox.x + tgtBox.width / 2,
    y: tgtBox.y + tgtBox.height / 2,
  };

  await page.mouse.move(srcCenter.x, srcCenter.y);
  await page.mouse.down();
  await page.mouse.move(srcCenter.x + 5, srcCenter.y + 5, { steps: 3 });
  await page.waitForTimeout(200);
  await page.mouse.move(tgtCenter.x, tgtCenter.y, { steps: 10 });
  await page.waitForTimeout(100);
  await page.mouse.up();
  await page.waitForTimeout(500);
}

test.describe("Drag and Drop", () => {
  let authToken: string;
  let email: string;

  test.beforeEach(async ({ page, rpc }) => {
    email = `${uniqueId("dnd")}@example.com`;
    const result = (await rpc("register", {
      name: "DnD User",
      email,
      password: "password123",
    })) as { access_token: string };
    authToken = result.access_token;
  });

  test("drop zones hidden before drag, visible during drag", async ({
    page,
  }) => {
    await createTasksViaApi(authToken, [
      { title: "Zone check", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 10_000 });

    // Drop zones should be in the DOM but hidden (display:none)
    const dropZones = column(page, COL.inbox).locator(".drop-zone");
    await expect(dropZones).toHaveCount(1, { timeout: 5_000 });
    await expect(dropZones.first()).not.toBeVisible();
  });

  test("drag single task from Inbox to Up next", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Move me", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Move me"),
      column(page, COL.upNext),
    );

    // Optimistic update: visible immediately
    await expect(
      column(page, COL.upNext).locator(".task-card-title", {
        hasText: "Move me",
      }),
    ).toBeVisible({ timeout: 5_000 });
    await expect(cards(page, COL.inbox)).toHaveCount(0, { timeout: 5_000 });
  });

  test("drag single task from Inbox to Paused", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Pause me", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Pause me"),
      column(page, COL.paused),
    );

    await expect(
      column(page, COL.paused).locator(".task-card-title", {
        hasText: "Pause me",
      }),
    ).toBeVisible({ timeout: 5_000 });
    await expect(cards(page, COL.inbox)).toHaveCount(0, { timeout: 5_000 });
  });

  test("drag task back from Up next to Inbox", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Bounce back", status: "up_next" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.upNext)).toHaveCount(1, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Bounce back"),
      column(page, COL.inbox),
    );

    await expect(
      column(page, COL.inbox).locator(".task-card-title", {
        hasText: "Bounce back",
      }),
    ).toBeVisible({ timeout: 5_000 });
  });

  test("drag 2nd task in column works reliably", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "First task", status: "inbox" },
      { title: "Second task", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(2, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Second task"),
      column(page, COL.upNext),
    );

    await expect(
      column(page, COL.upNext).locator(".task-card-title", {
        hasText: "Second task",
      }),
    ).toBeVisible({ timeout: 5_000 });
    await expect(
      column(page, COL.inbox).locator(".task-card-title", {
        hasText: "First task",
      }),
    ).toBeVisible();
  });

  test("drag 3rd task in column works reliably", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Alpha", status: "inbox" },
      { title: "Beta", status: "inbox" },
      { title: "Gamma", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(3, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Gamma"),
      column(page, COL.paused),
    );

    await expect(
      column(page, COL.paused).locator(".task-card-title", {
        hasText: "Gamma",
      }),
    ).toBeVisible({ timeout: 5_000 });
    await expect(cards(page, COL.inbox)).toHaveCount(2, { timeout: 5_000 });
  });

  test("consecutive drags from same column all succeed", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Batch A", status: "inbox" },
      { title: "Batch B", status: "inbox" },
      { title: "Batch C", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(3, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Batch A"),
      column(page, COL.upNext),
    );
    await expect(
      column(page, COL.upNext).locator(".task-card-title", {
        hasText: "Batch A",
      }),
    ).toBeVisible({ timeout: 5_000 });

    await dragCard(
      page,
      cardByTitle(page, "Batch B"),
      column(page, COL.upNext),
    );
    await expect(
      column(page, COL.upNext).locator(".task-card-title", {
        hasText: "Batch B",
      }),
    ).toBeVisible({ timeout: 5_000 });

    await dragCard(
      page,
      cardByTitle(page, "Batch C"),
      column(page, COL.paused),
    );
    await expect(
      column(page, COL.paused).locator(".task-card-title", {
        hasText: "Batch C",
      }),
    ).toBeVisible({ timeout: 5_000 });

    await expect(cards(page, COL.inbox)).toHaveCount(0, { timeout: 5_000 });
    await expect(cards(page, COL.upNext)).toHaveCount(2, { timeout: 5_000 });
    await expect(cards(page, COL.paused)).toHaveCount(1, { timeout: 5_000 });
  });

  test("dragged card gets dragging visual state", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Visible drag", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 10_000 });

    const card = cardByTitle(page, "Visible drag");
    const srcBox = (await card.boundingBox())!;

    await page.mouse.move(
      srcBox.x + srcBox.width / 2,
      srcBox.y + srcBox.height / 2,
    );
    await page.mouse.down();
    await page.mouse.move(
      srcBox.x + srcBox.width / 2 + 20,
      srcBox.y + 20,
      { steps: 5 },
    );
    await page.waitForTimeout(300);

    await expect(card).toHaveClass(/task-card-dragging/);

    // Drop zones should become visible during drag
    const zones = column(page, COL.inbox).locator(".drop-zone");
    await expect(zones.first()).toBeVisible({ timeout: 3_000 });

    await page.mouse.up();
  });

  test("drop zones in other columns activate during drag", async ({
    page,
  }) => {
    await createTasksViaApi(authToken, [
      { title: "Cross col", status: "inbox" },
      { title: "Target col task", status: "up_next" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 10_000 });
    await expect(cards(page, COL.upNext)).toHaveCount(1, { timeout: 10_000 });

    const card = cardByTitle(page, "Cross col");
    const srcBox = (await card.boundingBox())!;

    await page.mouse.move(
      srcBox.x + srcBox.width / 2,
      srcBox.y + srcBox.height / 2,
    );
    await page.mouse.down();
    await page.mouse.move(
      srcBox.x + srcBox.width / 2 + 10,
      srcBox.y + 10,
      { steps: 3 },
    );
    await page.waitForTimeout(200);

    const upNextZones = column(page, COL.upNext).locator(".drop-zone");
    await expect(upNextZones.first()).toBeVisible({ timeout: 3_000 });

    await page.mouse.up();
  });

  test("drag between different columns preserves tasks", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "In inbox", status: "inbox" },
      { title: "In up next", status: "up_next" },
      { title: "In paused", status: "paused" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "In up next"),
      column(page, COL.paused),
    );

    await expect(
      column(page, COL.paused).locator(".task-card-title", {
        hasText: "In up next",
      }),
    ).toBeVisible({ timeout: 5_000 });
    await expect(cards(page, COL.paused)).toHaveCount(2, { timeout: 5_000 });
    await expect(cards(page, COL.inbox)).toHaveCount(1, { timeout: 5_000 });
  });

  test("drag preserves other tasks in source column", async ({ page }) => {
    await createTasksViaApi(authToken, [
      { title: "Stay here 1", status: "inbox" },
      { title: "Stay here 2", status: "inbox" },
      { title: "I will move", status: "inbox" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(3, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "I will move"),
      column(page, COL.upNext),
    );

    await expect(
      column(page, COL.upNext).locator(".task-card-title", {
        hasText: "I will move",
      }),
    ).toBeVisible({ timeout: 5_000 });
    await expect(
      column(page, COL.inbox).locator(".task-card-title", {
        hasText: "Stay here 1",
      }),
    ).toBeVisible();
    await expect(
      column(page, COL.inbox).locator(".task-card-title", {
        hasText: "Stay here 2",
      }),
    ).toBeVisible();
    await expect(cards(page, COL.inbox)).toHaveCount(2);
  });

  test("consecutive drags across columns do not break state", async ({
    page,
  }) => {
    await createTasksViaApi(authToken, [
      { title: "Rapid A", status: "inbox" },
      { title: "Rapid B", status: "inbox" },
      { title: "Rapid C", status: "up_next" },
    ]);
    await loginAndWait(page, email, "password123");

    await expect(cards(page, COL.inbox)).toHaveCount(2, { timeout: 10_000 });

    await dragCard(
      page,
      cardByTitle(page, "Rapid A"),
      column(page, COL.paused),
    );
    await expect(
      column(page, COL.paused).locator(".task-card-title", {
        hasText: "Rapid A",
      }),
    ).toBeVisible({ timeout: 5_000 });

    await dragCard(
      page,
      cardByTitle(page, "Rapid B"),
      column(page, COL.upNext),
    );
    await expect(
      column(page, COL.upNext).locator(".task-card-title", {
        hasText: "Rapid B",
      }),
    ).toBeVisible({ timeout: 5_000 });

    await dragCard(
      page,
      cardByTitle(page, "Rapid C"),
      column(page, COL.paused),
    );
    await expect(
      column(page, COL.paused).locator(".task-card-title", {
        hasText: "Rapid C",
      }),
    ).toBeVisible({ timeout: 5_000 });

    await expect(cards(page, COL.inbox)).toHaveCount(0, { timeout: 5_000 });
    await expect(cards(page, COL.upNext)).toHaveCount(1, { timeout: 5_000 });
    await expect(cards(page, COL.paused)).toHaveCount(2, { timeout: 5_000 });
  });
});
