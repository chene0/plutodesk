import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for E2E testing
 * 
 * This configuration supports:
 * - Tauri desktop app testing
 * - Future backend API testing
 * - Full-stack integration testing
 */
export default defineConfig({
  // Test directory - currently set to tauri tests
  testDir: './tauri',
  
  // Maximum time one test can run for
  timeout: 30 * 1000,
  
  // Test execution settings
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : 1, // Run tests serially for Tauri app
  
  // Reporter configuration
  reporter: 'html',
  
  // Shared settings for all projects
  use: {
    // Base URL for the app
    baseURL: 'http://localhost:3000',
    
    // Screenshot and video on failure
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    
    // Trace on failure
    trace: 'on-first-retry',
  },

  // Configure projects for different test types
  projects: [
    {
      name: 'tauri',
      use: {
        ...devices['Desktop Chrome'],
        // Tauri-specific launcher
        launchOptions: {
          args: ['--disable-web-security'],
        },
      },
    },
  ],

  // Web server configuration - launches Tauri app for testing
  webServer: {
    command: 'cd ../frontend && npm run tauri:dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000, // Tauri build can take time
    stdout: 'ignore',
    stderr: 'pipe',
  },
});

