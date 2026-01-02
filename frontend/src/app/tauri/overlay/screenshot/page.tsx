"use client"

import { ScreenshotOverlay } from "@/components/screenshot-overlay";
import { closeScreenshotOverlay } from "@/utils/tauri/overlay/close_screenshot";

export default function Page() {
    return (
        <ScreenshotOverlay onClose={() => {
            console.log("Screenshot overlay closed");
            closeScreenshotOverlay();
        }} />
    );
}
