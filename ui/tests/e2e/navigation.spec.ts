import { test, expect } from '@playwright/test';
import {
  waitForFlutter,
  waitForSemanticsText,
  skipSplash,
  snapshot,
  clickFlutterWidget,
} from '../helpers/flutter_wait';

test.describe('Navigation rail', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await skipSplash(page);   // wait for splash to auto-transition
  });

  test('shows Bounty Market by default', async ({ page }) => {
    await waitForSemanticsText(page, 'Bounty Market');
    await waitForSemanticsText(page, 'Browse and trade anonymous social policy bonds');
  });

  test('nav rail shows all four destinations', async ({ page }) => {
    await waitForSemanticsText(page, 'BOUNTIES');
    await waitForSemanticsText(page, 'ISSUE');
    await waitForSemanticsText(page, 'WALLET');
    await waitForSemanticsText(page, 'NODE');
  });

  test('navigates to Issue Bounty', async ({ page }) => {
    await clickFlutterWidget(page, 'ISSUE');
    await waitForSemanticsText(page, 'Issue Bounty');
    await waitForSemanticsText(page, 'Define a social goal and issue bounties');
  });

  test('navigates to Wallet', async ({ page }) => {
    await clickFlutterWidget(page, 'WALLET');
    await waitForSemanticsText(page, 'Wallet');
    await waitForSemanticsText(page, 'anonymous bounty notes');
  });

  test('navigates to Node Status', async ({ page }) => {
    await clickFlutterWidget(page, 'NODE');
    await waitForSemanticsText(page, 'Node Status');
  });

  test('visual snapshot of main shell', async ({ page }) => {
    await waitForSemanticsText(page, 'Bounty Market');
    await snapshot(page, 'main-shell-bounties');
  });

  test('visual snapshot of Issue screen', async ({ page }) => {
    await clickFlutterWidget(page, 'ISSUE');
    await waitForSemanticsText(page, 'Issue Bounty');
    await snapshot(page, 'main-shell-issue');
  });
});
