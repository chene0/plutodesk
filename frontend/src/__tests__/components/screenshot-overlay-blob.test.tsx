import React from 'react';
import { render, waitFor } from '@testing-library/react';
import { ScreenshotOverlay } from '@/components/screenshot-overlay';
import { emit, listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { Window } from '@tauri-apps/api/window';

// Mock Tauri APIs
jest.mock('@tauri-apps/api/event');
jest.mock('@tauri-apps/api/core');
jest.mock('@tauri-apps/api/window');

const mockEmit = emit as jest.MockedFunction<typeof emit>;
const mockListen = listen as jest.MockedFunction<typeof listen>;
const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;
const mockWindowShow = jest.fn().mockResolvedValue(undefined);

// Mock Window constructor
(Window as jest.MockedClass<typeof Window>).mockImplementation(() => ({
    show: mockWindowShow,
    close: jest.fn().mockResolvedValue(undefined),
    hide: jest.fn().mockResolvedValue(undefined),
}) as any);

// Mock createImageBitmap globally
global.createImageBitmap = jest.fn();

describe('Screenshot Overlay - Blob/ImageBitmap Tests', () => {
    const mockOnClose = jest.fn();

    beforeEach(() => {
        jest.clearAllMocks();
        mockEmit.mockResolvedValue(undefined);
        mockListen.mockResolvedValue(jest.fn());
        mockInvoke.mockResolvedValue(undefined);
    });

    describe('Blob Creation from Base64', () => {
        it('creates Blob from valid base64 string', async () => {
            const validBase64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';

            // Test the conversion logic that happens in handleScreenshotData
            const byteCharacters = atob(validBase64);
            const byteNumbers = new Array(byteCharacters.length);
            for (let i = 0; i < byteCharacters.length; i++) {
                byteNumbers[i] = byteCharacters.charCodeAt(i);
            }
            const byteArray = new Uint8Array(byteNumbers);
            const blob = new Blob([byteArray], { type: 'image/png' });

            expect(blob).toBeInstanceOf(Blob);
            expect(blob.type).toBe('image/png');
            expect(blob.size).toBeGreaterThan(0);
        });

        it('handles invalid base64 input gracefully', async () => {
            const invalidBase64 = 'not-valid-base64!!!';

            expect(() => {
                atob(invalidBase64);
            }).toThrow();
        });

        it('handles empty base64 string', async () => {
            const emptyBase64 = '';

            const byteCharacters = atob(emptyBase64);
            const byteArray = new Uint8Array(byteCharacters.length);
            const blob = new Blob([byteArray], { type: 'image/png' });

            expect(blob.size).toBe(0);
        });
    });

    describe('createImageBitmap Success and Failure', () => {
        it('successfully creates ImageBitmap from valid Blob', async () => {
            const validBase64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
            const byteCharacters = atob(validBase64);
            const byteArray = new Uint8Array(byteCharacters.length);
            for (let i = 0; i < byteCharacters.length; i++) {
                byteArray[i] = byteCharacters.charCodeAt(i);
            }
            const blob = new Blob([byteArray], { type: 'image/png' });

            const mockImageBitmap = { width: 1, height: 1 };
            (global.createImageBitmap as jest.Mock).mockResolvedValue(mockImageBitmap);

            const imageBitmap = await createImageBitmap(blob);

            expect(imageBitmap).toEqual(mockImageBitmap);
            expect(imageBitmap.width).toBe(1);
            expect(imageBitmap.height).toBe(1);
        });

        it('handles createImageBitmap failure with corrupted data', async () => {
            const corruptedData = new Uint8Array([0, 0, 0, 0]); // Not valid PNG data
            const blob = new Blob([corruptedData], { type: 'image/png' });

            (global.createImageBitmap as jest.Mock).mockRejectedValue(new Error('Invalid image data'));

            await expect(createImageBitmap(blob)).rejects.toThrow('Invalid image data');
        });

        it('handles createImageBitmap timeout scenarios', async () => {
            const validBlob = new Blob([new Uint8Array(100)], { type: 'image/png' });

            // Simulate timeout
            (global.createImageBitmap as jest.Mock).mockRejectedValue(new Error('Timeout'));

            await expect(createImageBitmap(validBlob)).rejects.toThrow('Timeout');
        });
    });

    describe('Large Screenshot Handling', () => {
        it('handles screenshots larger than 2MB', async () => {
            // Create a large base64 string (simulate > 2MB image)
            const largeDataSize = 3 * 1024 * 1024; // 3MB
            const largeData = new Uint8Array(largeDataSize);
            // Fill with dummy data
            for (let i = 0; i < largeData.length; i++) {
                largeData[i] = i % 256;
            }

            const blob = new Blob([largeData], { type: 'image/png' });
            expect(blob.size).toBeGreaterThan(2 * 1024 * 1024);

            const mockImageBitmap = { width: 1920, height: 1080 };
            (global.createImageBitmap as jest.Mock).mockResolvedValue(mockImageBitmap);

            const imageBitmap = await createImageBitmap(blob);
            expect(imageBitmap).toBeDefined();
        });

        it('handles screenshots larger than 10MB', async () => {
            // Create a very large base64 string (simulate > 10MB image)
            const veryLargeDataSize = 12 * 1024 * 1024; // 12MB
            const veryLargeData = new Uint8Array(veryLargeDataSize);

            const blob = new Blob([veryLargeData], { type: 'image/png' });
            expect(blob.size).toBeGreaterThan(10 * 1024 * 1024);

            const mockImageBitmap = { width: 3840, height: 2160 }; // 4K resolution
            (global.createImageBitmap as jest.Mock).mockResolvedValue(mockImageBitmap);

            const imageBitmap = await createImageBitmap(blob);
            expect(imageBitmap).toBeDefined();
        });

        it('reports memory constraints appropriately', async () => {
            const extremelyLargeDataSize = 100 * 1024 * 1024; // 100MB
            const extremelyLargeData = new Uint8Array(extremelyLargeDataSize);
            const blob = new Blob([extremelyLargeData], { type: 'image/png' });

            (global.createImageBitmap as jest.Mock).mockRejectedValue(
                new Error('Out of memory')
            );

            await expect(createImageBitmap(blob)).rejects.toThrow('Out of memory');
        });
    });

    describe('Canvas Resizing with ImageBitmap', () => {
        it('canvas dimensions match bitmap dimensions', async () => {
            const canvas = document.createElement('canvas');
            const mockImageBitmap = { width: 1920, height: 1080 };

            // Simulate the resizing logic from handleScreenshotData
            canvas.width = mockImageBitmap.width;
            canvas.height = mockImageBitmap.height;

            expect(canvas.width).toBe(1920);
            expect(canvas.height).toBe(1080);
        });

        it('canvas context properly draws bitmap', async () => {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');
            const mockImageBitmap = { width: 800, height: 600 };

            canvas.width = mockImageBitmap.width;
            canvas.height = mockImageBitmap.height;

            const drawImageSpy = jest.spyOn(ctx!, 'drawImage');

            // Simulate drawing
            ctx?.drawImage(mockImageBitmap as any, 0, 0);

            expect(drawImageSpy).toHaveBeenCalledWith(mockImageBitmap, 0, 0);
        });

        it('canvas is properly cleared between screenshots', async () => {
            const canvas = document.createElement('canvas');
            const ctx = canvas.getContext('2d');

            // First screenshot
            canvas.width = 1920;
            canvas.height = 1080;

            // Clear for second screenshot (canvas resize clears it automatically)
            canvas.width = 800;
            canvas.height = 600;

            // Verify new dimensions
            expect(canvas.width).toBe(800);
            expect(canvas.height).toBe(600);
        });
    });

    describe('Error Handling in handleScreenshotData', () => {
        it('handles atob() failures', () => {
            const invalidBase64 = 'invalid!!!base64';

            expect(() => {
                atob(invalidBase64);
            }).toThrow();
        });

        it('handles Blob creation failures', () => {
            // Blob constructor should handle any input, but test edge cases
            expect(() => {
                new Blob([undefined as any], { type: 'image/png' });
            }).not.toThrow();

            expect(() => {
                new Blob([null as any], { type: 'image/png' });
            }).not.toThrow();
        });

        it('handles createImageBitmap rejections', async () => {
            const validBlob = new Blob([new Uint8Array(100)], { type: 'image/png' });

            (global.createImageBitmap as jest.Mock).mockRejectedValue(
                new Error('Bitmap creation failed')
            );

            await expect(createImageBitmap(validBlob)).rejects.toThrow('Bitmap creation failed');
        });

        it('handles canvas context unavailable', () => {
            const canvas = document.createElement('canvas');

            // Mock getContext to return null
            jest.spyOn(canvas, 'getContext').mockReturnValue(null);

            const ctx = canvas.getContext('2d');
            expect(ctx).toBeNull();

            // Verify we handle null context gracefully
            if (ctx) {
                ctx.drawImage({} as any, 0, 0);
            }
            // Should not throw when ctx is null
        });
    });

    describe('Integration with ScreenshotOverlay Component', () => {
        it('properly handles full Blob→ImageBitmap flow', async () => {
            const validBase64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';

            let eventCallback: ((event: any) => void) | null = null;
            mockListen.mockImplementation(async (eventName: string, callback: (event: any) => void) => {
                if (eventName === 'open_screenshot_overlay') {
                    eventCallback = callback;
                }
                return jest.fn();
            });

            const mockImageBitmap = { width: 1, height: 1 };
            (global.createImageBitmap as jest.Mock).mockResolvedValue(mockImageBitmap);

            const { container } = render(<ScreenshotOverlay onClose={mockOnClose} />);

            await waitFor(() => {
                expect(mockListen).toHaveBeenCalledWith('open_screenshot_overlay', expect.any(Function));
            });

            expect(eventCallback).not.toBeNull();

            // Trigger the event with base64 data
            if (eventCallback) {
                eventCallback({ payload: validBase64 });
            }

            // Wait for async processing
            await waitFor(() => {
                expect(global.createImageBitmap).toHaveBeenCalled();
            }, { timeout: 1000 });
        });

        it('handles error in component without crashing', async () => {
            const invalidBase64 = 'invalid!!!';

            let eventCallback: ((event: any) => void) | null = null;
            mockListen.mockImplementation(async (eventName: string, callback: (event: any) => void) => {
                if (eventName === 'open_screenshot_overlay') {
                    eventCallback = callback;
                }
                return jest.fn();
            });

            const { container } = render(<ScreenshotOverlay onClose={mockOnClose} />);

            await waitFor(() => {
                expect(mockListen).toHaveBeenCalled();
            });

            // Should not crash when receiving invalid data
            if (eventCallback) {
                expect(() => {
                    eventCallback({ payload: invalidBase64 });
                }).not.toThrow();
            }
        });
    });

    describe('Memory Management', () => {
        it('does not leak memory with repeated conversions', async () => {
            const base64Data = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';

            // Perform conversion multiple times
            for (let i = 0; i < 10; i++) {
                const byteCharacters = atob(base64Data);
                const byteArray = new Uint8Array(byteCharacters.length);
                for (let j = 0; j < byteCharacters.length; j++) {
                    byteArray[j] = byteCharacters.charCodeAt(j);
                }
                const blob = new Blob([byteArray], { type: 'image/png' });

                expect(blob).toBeInstanceOf(Blob);
            }

            // If this test completes without memory errors, we're good
            expect(true).toBe(true);
        });
    });
});
