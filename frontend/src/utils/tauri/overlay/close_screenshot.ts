import { invoke } from '@tauri-apps/api/core';

export function closeScreenshotOverlay() {
    invoke('close_screenshot_overlay');
}
