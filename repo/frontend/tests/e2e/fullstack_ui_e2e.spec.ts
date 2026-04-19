import { test, expect } from "@playwright/test";

test.describe("FleetReserve fullstack UI e2e", () => {
  test("login and key navigation flow", async ({ page }) => {
    await page.goto("http://localhost:8081/login");

    await expect(page.getByText("FleetReserve Login")).toBeVisible();
    await page.getByLabel("Username").fill("admin");
    await page.getByLabel("Password").fill("FleetReserveHttpTest#2026");
    await page.getByRole("button", { name: "Sign In" }).click();

    // After successful login, dashboard and nav should be visible.
    await expect(page.getByText("Dashboard")).toBeVisible();
    await expect(page.getByRole("link", { name: "Vehicles" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Calendar" })).toBeVisible();

    // Navigate to core pages to validate FE↔BE wiring through UI.
    await page.getByRole("link", { name: "Vehicles" }).click();
    await expect(page.getByText("Vehicle Management")).toBeVisible();

    await page.getByRole("link", { name: "Reservations" }).click();
    await expect(page.getByText("Reservations")).toBeVisible();

    await page.getByRole("link", { name: "Admin" }).click();
    await expect(page.getByText("Administration")).toBeVisible();
  });

  test("calendar supports day/week toggle and status filtering", async ({ page }) => {
    await page.goto("http://localhost:8081/login");
    await page.getByLabel("Username").fill("admin");
    await page.getByLabel("Password").fill("FleetReserveHttpTest#2026");
    await page.getByRole("button", { name: "Sign In" }).click();

    await page.getByRole("link", { name: "Calendar" }).click();
    await expect(page.getByText("Availability Calendar")).toBeVisible();

    // Day mode should be available by default.
    await expect(page.getByRole("button", { name: "Day" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Week" })).toBeVisible();

    // Switch to week mode and check week grid container exists.
    await page.getByRole("button", { name: "Week" }).click();
    await expect(page.locator(".calendar-grid-week")).toBeVisible();

    // Toggle a status filter to ensure filter controls are interactive.
    const inRepair = page.getByLabel("in-repair");
    await inRepair.check();
    await expect(inRepair).toBeChecked();

    // Switch back to day mode and ensure the base calendar grid remains visible.
    await page.getByRole("button", { name: "Day" }).click();
    await expect(page.locator(".calendar-grid").first()).toBeVisible();
  });
});
