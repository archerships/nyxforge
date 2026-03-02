import { test, expect } from '@playwright/test';
import { waitForFlutter, waitForSemanticsText, snapshot } from '../helpers/flutter_wait';

test.describe('Splash screen', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForFlutter(page);
  });

  test('renders NYXFORGE title', async ({ page }) => {
    await waitForSemanticsText(page, 'NYXFORGE');
  });

  test('renders tagline', async ({ page }) => {
    await waitForSemanticsText(page, 'anonymous social policy bond market');
  });

  test('transitions to main shell automatically', async ({ page }) => {
    // Splash animates for ~900ms then waits 600ms before transition
    await page.waitForTimeout(3_000);
    // After transition the Bond Market screen should be visible
    await waitForSemanticsText(page, 'Bond Market', 8_000);
  });

  test('visual snapshot', async ({ page }) => {
    // Capture the splash at peak opacity (after fade-in, before transition)
    await page.waitForTimeout(1_000);
    await snapshot(page, 'splash-screen');
  });

  test('no external network calls during boot', async ({ page }) => {
    const externalRequests: string[] = [];

    page.on('request', (req) => {
      const url = req.url();
      // Allow localhost; flag anything else
      if (!url.startsWith('http://localhost') && !url.startsWith('http://127.0.0.1')) {
        externalRequests.push(url);
      }
    });

    await page.goto('/');
    await waitForFlutter(page);
    await page.waitForTimeout(2_000);

    // Filter out known-acceptable external calls (none expected in de-googled build)
    const unexpected = externalRequests.filter(
      (u) => !u.includes('localhost') && !u.includes('127.0.0.1'),
    );

    expect(unexpected, `Unexpected external requests: ${unexpected.join(', ')}`).toHaveLength(0);
  });
});
