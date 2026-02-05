import { test, expect } from '@playwright/test';

test.describe('Visual States', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for app to be fully loaded
    await page.waitForSelector('[class*="card"]');
  });

  test('idle state appearance', async ({ page }) => {
    await expect(page).toHaveScreenshot('idle-state.png', {
      maxDiffPixelRatio: 0.01,
    });
  });

  test('controls panel appearance', async ({ page }) => {
    const controlsCard = page.locator('[class*="card"]').first();
    await expect(controlsCard).toHaveScreenshot('controls-panel.png', {
      maxDiffPixelRatio: 0.01,
    });
  });

  test('output panel appearance', async ({ page }) => {
    const outputCard = page.locator('[class*="card"]').last();
    await expect(outputCard).toHaveScreenshot('output-panel.png', {
      maxDiffPixelRatio: 0.01,
    });
  });
});
