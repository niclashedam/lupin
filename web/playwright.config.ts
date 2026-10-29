import { defineConfig } from "@playwright/test";

// Runs against the production build served by Wrangler, so the _headers CSP applies.
export default defineConfig({
  testDir: "e2e",
  use: { baseURL: "http://localhost:8788" },
  webServer: { command: "npx wrangler dev --port 8788", url: "http://localhost:8788", reuseExistingServer: !process.env.CI },
});
