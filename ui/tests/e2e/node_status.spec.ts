import { test, expect } from '@playwright/test';
import {
  waitForFlutter,
  waitForSemanticsText,
  skipSplash,
  snapshot,
  clickFlutterWidget,
} from '../helpers/flutter_wait';

test.describe('Node status screen', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await skipSplash(page);
    await clickFlutterWidget(page, 'NODE');
    await waitForSemanticsText(page, 'Node Status');
  });

  test('shows offline error when node is unreachable', async ({ page }) => {
    // The local nyxforge-node is not running during tests; expect offline UI
    await page.route('http://127.0.0.1:8888/rpc', (route) =>
      route.abort('connectionrefused'),
    );
    // Trigger a refresh
    await clickFlutterWidget(page, 'REFRESH');
    await waitForSemanticsText(page, 'Node offline', 8_000);
  });

  test('shows connected status when node responds', async ({ page }) => {
    // Mock a healthy node response
    await page.route('http://127.0.0.1:8888/rpc', (route) =>
      route.fulfill({
        status:      200,
        contentType: 'application/json',
        body: JSON.stringify({
          result: {
            version:    '0.1.0',
            bonds:      42,
          },
        }),
      }),
    );

    await clickFlutterWidget(page, 'REFRESH');
    await waitForSemanticsText(page, 'Connected',  8_000);
    await waitForSemanticsText(page, '0.1.0');
    await waitForSemanticsText(page, '42');
  });

  test('shows RPC endpoint', async ({ page }) => {
    await waitForSemanticsText(page, '127.0.0.1:8888');
  });

  test('visual snapshot - node offline', async ({ page }) => {
    await page.route('http://127.0.0.1:8888/rpc', (route) => route.abort());
    await clickFlutterWidget(page, 'REFRESH');
    await waitForSemanticsText(page, 'Node offline', 8_000);
    await snapshot(page, 'node-status-offline');
  });

  test('visual snapshot - node connected', async ({ page }) => {
    await page.route('http://127.0.0.1:8888/rpc', (route) =>
      route.fulfill({
        status:      200,
        contentType: 'application/json',
        body: JSON.stringify({ result: { version: '0.1.0', bonds: 7 } }),
      }),
    );
    await clickFlutterWidget(page, 'REFRESH');
    await waitForSemanticsText(page, 'Connected', 8_000);
    await snapshot(page, 'node-status-connected');
  });
});

test.describe('Node RPC security', () => {
  test('RPC endpoint only contacts localhost', async ({ page }) => {
    const rpcTargets: string[] = [];

    page.on('request', (req) => {
      if (req.url().includes('/rpc')) {
        rpcTargets.push(req.url());
      }
    });

    await page.goto('/');
    await skipSplash(page);
    await clickFlutterWidget(page, 'NODE');
    await page.waitForTimeout(2_000);

    // All RPC calls must go to localhost only
    for (const url of rpcTargets) {
      expect(
        url.startsWith('http://127.0.0.1') || url.startsWith('http://localhost'),
        `RPC call to non-localhost: ${url}`,
      ).toBe(true);
    }
  });
});
