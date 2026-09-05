import { test, expect } from '@playwright/test';

test('has title', async ({ page }) => {
  await page.goto('/');

  await expect(page).toHaveTitle(/Ledger Oxide/);
});

test('page looks correct', async ({ page }) => {
  await page.goto('/');

  await expect(page).toHaveScreenshot();
});
