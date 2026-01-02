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

