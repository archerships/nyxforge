/**
 * flutter_wait.ts -- Helpers for waiting on Flutter WASM boot and semantics.
 *
 * Flutter web renders to a <canvas> and builds a parallel accessibility tree
 * via <flt-semantics> elements (shadow DOM). These helpers abstract that away.
 *
 * Boot sequence we wait for:
 *   1. <flt-glass-pane> appears  -- Flutter engine mounted
 *   2. Shadow root contains content -- first frame rendered
 *   3. Specific semantic label visible -- target widget ready
 */

import { type Page, type Locator, expect } from '@playwright/test';

// The custom elements Flutter injects into the page
const FLUTTER_PANE      = 'flt-glass-pane';
const FLUTTER_SEMANTICS = 'flt-semantics';

/**
 * Wait for the Flutter engine to mount and render at least one frame.
 * Call this at the start of every Flutter test before any other assertion.
 */
export async function waitForFlutter(page: Page, timeout = 15_000): Promise<void> {
  // Step 1: Flutter glass pane appears (engine is running)
  await page.waitForSelector(FLUTTER_PANE, { timeout });

  // Step 2: Semantics tree is populated (first frame drawn)
  await page.waitForFunction(
    (sel) => {
      const pane = document.querySelector(sel);
      const root = pane?.shadowRoot;
      return root && root.querySelector('flt-semantics') !== null;
    },
    FLUTTER_PANE,
    { timeout },
  );
}

/**
 * Wait for a specific text label to appear in Flutter's semantics tree.
 * Use this instead of page.getByText() since Flutter renders to canvas.
 */
export async function waitForSemanticsText(
  page:    Page,
  text:    string,
  timeout = 10_000,
): Promise<void> {
  await page.waitForFunction(
    ({ pane, target }) => {
      const root = document.querySelector(pane)?.shadowRoot;
      if (!root) return false;
      const nodes = root.querySelectorAll('flt-semantics');
      return Array.from(nodes).some(
        (n) =>
          n.getAttribute('aria-label')?.includes(target) ||
          n.textContent?.includes(target),
      );
    },
    { pane: FLUTTER_PANE, target: text },
    { timeout },
  );
}

/**
 * Find a Flutter semantics element by aria-label.
 * Returns a Playwright Locator for further assertions.
 */
export function flutterLocator(page: Page, ariaLabel: string): Locator {
  // Pierce the shadow root of flt-glass-pane to reach flt-semantics nodes
  return page
    .locator(`${FLUTTER_PANE} >> internal:shadow=[aria-label="${ariaLabel}"]`);
}

/**
 * Take a named screenshot for visual regression comparison.
 * Screenshots are stored in tests/screenshots/.
 */
export async function snapshot(page: Page, name: string): Promise<void> {
  await expect(page).toHaveScreenshot(`${name}.png`, {
    maxDiffPixelRatio: 0.02,   // allow 2% pixel diff (anti-aliasing, subpixel)
    animations: 'disabled',
  });
}

/**
 * Click a Flutter widget identified by aria-label.
 * Uses JS evaluation to reach through shadow DOM.
 */
export async function clickFlutterWidget(page: Page, ariaLabel: string): Promise<void> {
  await page.evaluate(
    ({ pane, label }) => {
      const root = document.querySelector(pane)?.shadowRoot;
      const el = root?.querySelector(`[aria-label="${label}"]`) as HTMLElement | null;
      if (!el) throw new Error(`Widget with aria-label "${label}" not found`);
      el.click();
    },
    { pane: FLUTTER_PANE, label: ariaLabel },
  );
}

/**
 * Dismiss the splash screen by waiting for it to auto-transition.
 * The splash fades after ~1.5s; use this to get to the main shell quickly.
 */
export async function skipSplash(page: Page): Promise<void> {
  await waitForFlutter(page);
  // Splash auto-transitions after 900ms animation + 600ms pause
  await page.waitForTimeout(2_000);
}
