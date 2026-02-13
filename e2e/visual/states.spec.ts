import { type Page, expect, test } from '@playwright/test';

async function hasNoBackend(page: Page): Promise<boolean> {
  return await page.getByRole('heading', { name: 'No Backend Connection' }).isVisible();
}

test.describe('Visual States', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    if (!(await hasNoBackend(page))) {
      await page.waitForSelector('[class*="card"]');
    }
  });

  test('idle state appearance', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    await expect(page).toHaveScreenshot('idle-state.png', {
      maxDiffPixelRatio: 0.01,
    });
  });

  test('controls panel appearance', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    const controlsCard = page.locator('[class*="card"]').first();
    await expect(controlsCard).toHaveScreenshot('controls-panel.png', {
      maxDiffPixelRatio: 0.01,
    });
  });

  test('output panel appearance', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    const outputCard = page.locator('[class*="card"]').last();
    await expect(outputCard).toHaveScreenshot('output-panel.png', {
      maxDiffPixelRatio: 0.01,
    });
  });
});
