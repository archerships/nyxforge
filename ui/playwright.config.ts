import { defineConfig, devices } from '@playwright/test';

/**
 * Flutter WASM apps have a multi-stage boot:
 *   1. HTML loads (~50ms)
 *   2. flutter.js / bootstrap JS executes (~100ms)
 *   3. main.dart.wasm fetched and compiled (~500ms-2s depending on size)
 *   4. Dart main() runs, Flutter engine starts (~200ms)
 *   5. First frame rendered
 *
 * All timeouts below are generous to accommodate WASM compilation time.
 */

const DEV_SERVER_URL  = 'http://localhost:8080';
const PROD_BUILD_DIR  = 'build/web';

export default defineConfig({
  testDir:   './tests/e2e',
  outputDir: './tests/results',

  // Give each test plenty of time for WASM boot
  timeout:   30_000,
  expect:    { timeout: 15_000 },

  // Run all tests in each file in parallel
  fullyParallel: false,   // false: Flutter dev server is single-origin

  // Fail the build on test.only left in source
  forbidOnly: !!process.env.CI,

  // Retry on CI to handle flaky WASM startup
  retries: process.env.CI ? 2 : 0,

  // Single worker to avoid port conflicts with the Flutter server
  workers: 1,

  reporter: [
    ['list'],
    ['html', { outputFolder: 'tests/playwright-report', open: 'never' }],
  ],

  use: {
    baseURL: DEV_SERVER_URL,

    // Collect traces on first retry (great for debugging WASM failures)
    trace:      'on-first-retry',
    screenshot: 'only-on-failure',
    video:      'retain-on-failure',

    // COOP/COEP headers are required for SharedArrayBuffer used by WASM threads
    extraHTTPHeaders: {
      'Cross-Origin-Opener-Policy':   'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },

  projects: [
    {
      name: 'chromium',
      use:  {
        ...devices['Desktop Chrome'],
        // Enable WASM features: threads, SIMD, exception handling
        launchOptions: {
          args: [
            '--enable-features=WebAssemblyBaseline,WebAssemblyTiering',
            '--js-flags=--experimental-wasm-gc',
          ],
        },
      },
    },
    {
      name: 'firefox',
      use:  { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use:  { ...devices['Desktop Safari'] },
    },

    // Mobile viewports
    {
      name: 'mobile-chrome',
      use:  { ...devices['Pixel 7'] },
    },
  ],

  // Start Flutter dev server before running tests.
  // For CI, use the pre-built output instead: `flutter build web --wasm`
  // then serve it with a static server.
  webServer: {
    command: [
      'export PATH="$HOME/development/flutter/bin:$PATH"',
      '&& flutter run -d web-server',
      '--web-port=8080',
      '--web-hostname=localhost',
      '--release',
    ].join(' '),
    url:              DEV_SERVER_URL,
    reuseExistingServer: !process.env.CI,
    timeout:          60_000,   // Flutter first compile can be slow
    stdout:           'pipe',
    stderr:           'pipe',
  },
});
