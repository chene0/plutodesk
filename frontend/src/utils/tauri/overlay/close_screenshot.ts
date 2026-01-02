import { invoke } from '@tauri-apps/api/core';

export function closeScreenshotOverlay() {
    invoke('close_screenshot_overlay').catch((error) => {
        console.error('Failed to close screenshot overlay:', error);
    });
}
