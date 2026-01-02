import { renderHook, waitFor } from '@testing-library/react';
import { useTauriListeners } from '@/hooks/useTauriListeners';
import { listen } from '@tauri-apps/api/event';

jest.mock('@tauri-apps/api/event');

const mockListen = listen as jest.MockedFunction<typeof listen>;

describe('useTauriListeners', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('registers event listener on mount', async () => {
    const mockUnlisten = jest.fn();
    mockListen.mockResolvedValue(mockUnlisten);

    const callback = jest.fn();

    renderHook(() => useTauriListeners('test-event', callback));

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith('test-event', expect.any(Function));
    });
  });

  it('calls callback when event is received', async () => {
    const mockUnlisten = jest.fn();
    let eventCallback: ((event: any) => void) | null = null;

    mockListen.mockImplementation(async (eventName, callback) => {
      eventCallback = callback as (event: any) => void;
      return mockUnlisten;
    });

    const testCallback = jest.fn();

    renderHook(() => useTauriListeners('test-event', testCallback));

    await waitFor(() => {
      expect(eventCallback).not.toBeNull();
    });

    // Simulate event
    if (eventCallback) {
      const testEvent = { payload: 'test-data' };
      eventCallback(testEvent);
      expect(testCallback).toHaveBeenCalledWith(testEvent);
    }
  });

  it('unregisters listener on unmount', async () => {
    const mockUnlisten = jest.fn();
    mockListen.mockResolvedValue(mockUnlisten);

    const callback = jest.fn();

    const { unmount } = renderHook(() => useTauriListeners('test-event', callback));

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalled();
    });

    unmount();

    expect(mockUnlisten).toHaveBeenCalled();
  });

  it('handles multiple event registrations', async () => {
    const mockUnlisten1 = jest.fn();
    const mockUnlisten2 = jest.fn();
    mockListen
      .mockResolvedValueOnce(mockUnlisten1)
      .mockResolvedValueOnce(mockUnlisten2);

    const callback1 = jest.fn();
    const callback2 = jest.fn();

    const { unmount: unmount1 } = renderHook(() =>
      useTauriListeners('event-1', callback1)
    );
    const { unmount: unmount2 } = renderHook(() =>
      useTauriListeners('event-2', callback2)
    );

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledTimes(2);
    });

    unmount1();
    unmount2();

    expect(mockUnlisten1).toHaveBeenCalled();
    expect(mockUnlisten2).toHaveBeenCalled();
  });

  it('handles listener registration errors gracefully', async () => {
    const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();
    const testError = new Error('Registration failed');
    mockListen.mockRejectedValue(testError);

    const callback = jest.fn();

    // The hook should not throw even if listen fails
    expect(() => {
      renderHook(() => useTauriListeners('test-event', callback));
    }).not.toThrow();

    // Wait for the async operation to complete
    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith('test-event', expect.any(Function));
    }, { timeout: 1000 });

    // Wait a bit more for the error to be caught and logged
    await new Promise((resolve) => setTimeout(resolve, 10));

    // Verify error was logged
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Failed to register listener for event "test-event":',
      testError
    );

    // Verify cleanup function doesn't crash even if registration failed
    const { unmount } = renderHook(() => useTauriListeners('test-event-2', callback));
    await waitFor(() => {
      expect(mockListen).toHaveBeenCalled();
    });
    unmount(); // Should not throw even if unlisten is undefined

    consoleErrorSpy.mockRestore();
  });
});

