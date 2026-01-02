import { closeScreenshotOverlay } from '@/utils/tauri/overlay/close_screenshot';
import { invoke } from '@tauri-apps/api/core';

jest.mock('@tauri-apps/api/core');

const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;

describe('closeScreenshotOverlay', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it('invokes close_screenshot_overlay command', () => {
    closeScreenshotOverlay();

    expect(mockInvoke).toHaveBeenCalledWith('close_screenshot_overlay');
  });

  it('handles invocation errors gracefully', async () => {
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();
    const testError = new Error('Invocation failed');
    mockInvoke.mockRejectedValue(testError);

    // Should not throw synchronously
    expect(() => {
      closeScreenshotOverlay();
    }).not.toThrow();

    expect(mockInvoke).toHaveBeenCalledWith('close_screenshot_overlay');

    // Wait for the promise rejection to be handled
    await new Promise((resolve) => setTimeout(resolve, 10));

    // Verify error was logged
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Failed to close screenshot overlay:',
      testError
    );

    consoleErrorSpy.mockRestore();
  });
});

