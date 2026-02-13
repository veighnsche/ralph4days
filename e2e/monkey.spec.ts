import { type Page, expect, test } from '@playwright/test';

async function hasNoBackend(page: Page): Promise<boolean> {
  return await page.getByRole('heading', { name: 'No Backend Connection' }).isVisible();
}

test.describe('Chaos Testing', () => {
  test('app survives 500 random interactions', async ({ page }) => {
    const errors: string[] = [];

    // Capture console errors
    page.on('pageerror', (err) => {
      errors.push(err.message);
    });

    await page.goto('/');
    if (await hasNoBackend(page)) {
      await expect(page.getByRole('heading', { name: 'No Backend Connection' })).toBeVisible();
      return;
    }

    await page.waitForSelector('[class*="card"]');

    // Inject Gremlins.js
    await page.addScriptTag({
      url: 'https://unpkg.com/gremlins.js@2.2.0/dist/gremlins.min.js',
    });

    // Wait for gremlins to load
    await page.waitForFunction(() => typeof (window as any).gremlins !== 'undefined');

    // Configure and unleash gremlins
    await page.evaluate(() => {
      return new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('Gremlins timed out'));
        }, 60000);

        (window as any).gremlins
          .createHorde({
            species: [
              (window as any).gremlins.species.clicker({
                clickTypes: ['click'],
              }),
              (window as any).gremlins.species.formFiller(),
              (window as any).gremlins.species.scroller(),
              (window as any).gremlins.species.typer(),
            ],
            mogwais: [
              (window as any).gremlins.mogwais.gizmo(),
            ],
            strategies: [
              (window as any).gremlins.strategies.distribution({
                distribution: [0.4, 0.2, 0.2, 0.2],
                delay: 50,
              }),
            ],
          })
          .unleash({ nb: 500 })
          .then(() => {
            clearTimeout(timeout);
            resolve();
          })
          .catch((err: Error) => {
            clearTimeout(timeout);
            reject(err);
          });
      });
    });

    // Wait a moment for any async errors
    await page.waitForTimeout(1000);

    // Verify app didn't crash - cards should still be visible
    await expect(page.locator('[class*="card"]').first()).toBeVisible();

    // Report any errors (but don't fail - some errors might be expected)
    if (errors.length > 0) {
      console.warn('Console errors during monkey testing:', errors);
    }

    // Critical errors should fail the test
    const criticalErrors = errors.filter(
      (e) =>
        e.includes('Cannot read') ||
        e.includes('is not defined') ||
        e.includes('Maximum call stack')
    );
    expect(criticalErrors).toHaveLength(0);
  });
});
