import { type Page, expect, test } from '@playwright/test';

async function hasNoBackend(page: Page): Promise<boolean> {
  return await page.getByRole('heading', { name: 'No Backend Connection' }).isVisible();
}

test.describe('Loop Controls', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('displays idle status on load', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      await expect(
        page.getByText('Ralph4days requires the Tauri desktop runtime. It cannot run in a browser.')
      ).toBeVisible();
      return;
    }

    await expect(page.getByText('Idle')).toBeVisible();
  });

  test('start button disabled without project path', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    const startBtn = page.getByRole('button', { name: 'Start' });
    await expect(startBtn).toBeDisabled();
  });

  test('start button enabled with project path', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    const input = page.locator('input[placeholder*="path"]');
    await input.fill('/tmp/test-project');

    const startBtn = page.getByRole('button', { name: 'Start' });
    await expect(startBtn).toBeEnabled();
  });

  test('pause and resume buttons disabled when idle', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    await expect(page.getByRole('button', { name: 'Pause' })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Resume' })).toBeDisabled();
  });

  test('stop button disabled when idle', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    await expect(page.getByRole('button', { name: 'Stop' })).toBeDisabled();
  });

  test('max iterations input accepts numbers', async ({ page }) => {
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    const input = page.locator('input[type="number"]');
    await input.fill('50');
    await expect(input).toHaveValue('50');
  });
});
