import { defineConfig, devices } from "@playwright/test";

const TEST_URL = process.env.FORGE_TEST_URL;

export default defineConfig({
  testDir: "./tests",
  // Keep Playwright output outside the frontend dir so dx serve's file watcher
  // doesn't trigger a rebuild mid-test.
  outputDir: "../test-results",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 1,
  timeout: 180_000,
  workers: process.env.CI ? 1 : undefined,
  reporter: [["html", { outputFolder: "../playwright-report" }]],
  globalSetup: "./tests/global-setup.ts",
  use: {
    baseURL: TEST_URL || "http://localhost:9080",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  ...(TEST_URL
    ? {}
    : {
        webServer: {
          command:
            "dx build --web --release && npx serve target/dx/kaizen-frontend/release/web/public -l 9080 --cors --no-port-switching --single",
          url: "http://localhost:9080",
          reuseExistingServer: true,
          timeout: 120 * 1000,
        },
      }),
});
