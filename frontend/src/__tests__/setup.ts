import '@testing-library/jest-dom';

// Mock Tauri APIs
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

jest.mock('@tauri-apps/api/event', () => ({
  listen: jest.fn(),
  emit: jest.fn(),
}));

jest.mock('@tauri-apps/api/window', () => ({
  Window: jest.fn().mockImplementation(() => ({
    show: jest.fn().mockResolvedValue(undefined),
    close: jest.fn().mockResolvedValue(undefined),
    hide: jest.fn().mockResolvedValue(undefined),
  })),
}));

// ---------------------------------------------------------------------------
// JSDOM environment polyfills/mocks for browser image/canvas APIs used by UI
// ---------------------------------------------------------------------------

// createImageBitmap is not implemented in JSDOM.
// Individual tests may override this mock as needed.
if (typeof global.createImageBitmap === 'undefined') {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (global as any).createImageBitmap = jest.fn(async () => ({
    width: 1,
    height: 1,
  }));
}

// Canvas APIs are not implemented in JSDOM by default.
// Provide a minimal 2D context mock that supports our component/test usage.
const createMockCanvas2DContext = () => ({
  drawImage: jest.fn(),
  getImageData: jest.fn(() => ({ data: new Uint8ClampedArray(), width: 0, height: 0 })),
  putImageData: jest.fn(),
  clearRect: jest.fn(),
});

Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
  value: function getContext(this: HTMLCanvasElement, contextId: string) {
    if (contextId !== '2d') return null;
    return createMockCanvas2DContext();
  },
});

if (typeof HTMLCanvasElement.prototype.toDataURL !== 'function') {
  Object.defineProperty(HTMLCanvasElement.prototype, 'toDataURL', {
    value: () => 'data:image/png;base64,',
  });
}

