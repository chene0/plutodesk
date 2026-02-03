"use client";

import type React from "react";

import { useState, useRef, useCallback, useEffect } from "react";
import { X, Copy, Download, Crop } from "lucide-react";
import { useTauriListeners } from "@/hooks/useTauriListeners";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";

interface SelectionArea {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
}

interface ScreenshotOverlayProps {
  onClose: () => void;
}

export function ScreenshotOverlay({ onClose }: ScreenshotOverlayProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [isSelecting, setIsSelecting] = useState(false);
  const [selection, setSelection] = useState<SelectionArea | null>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  // Create Window instance inside component to avoid stale references
  // Use a ref to ensure we only create it once per component instance
  const appWindowRef = useRef<Window | null>(null);

  const getAppWindow = useCallback(() => {
    if (!appWindowRef.current) {
      try {
        appWindowRef.current = new Window("screenshot_overlay");
      } catch (error) {
        console.error("[screenshot-overlay] Failed to create Window instance:", error);
        // Return a mock object to prevent crashes
        return {
          show: async () => { },
          close: async () => { },
          hide: async () => { },
        } as Window;
      }
    }
    return appWindowRef.current;
  }, []);

  useEffect(() => {
    emit("screenshot_overlay_ready", { label: "screenshot_overlay" })
      .catch((err) => {
        console.error("[screenshot-overlay] Failed to emit ready event", err);
      });

    // Cleanup: clear window reference on unmount
    return () => {
      appWindowRef.current = null;
    };
  }, []);

  // Helper function to process screenshot data (used by both command and event)
  const handleScreenshotData = useCallback(async (base64: string) => {
    try {
      // Convert base64 to blob to avoid data URL size limits
      const byteCharacters = atob(base64);
      const byteNumbers = new Array(byteCharacters.length);
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i);
      }
      const byteArray = new Uint8Array(byteNumbers);
      const blob = new Blob([byteArray], { type: 'image/png' });

      // Create image bitmap from blob
      const imageBitmap = await createImageBitmap(blob);

      const canvas = canvasRef.current;

      if (canvas) {
        canvas.width = imageBitmap.width;
        canvas.height = imageBitmap.height;

        const ctx = canvas.getContext("2d");
        if (ctx) {
          ctx.drawImage(imageBitmap, 0, 0);
          try {
            const appWindow = getAppWindow();
            await appWindow.show();
          } catch (error) {
            console.error("[screenshot-overlay] Failed to show window:", error);
          }
        } else {
          console.warn("[screenshot-overlay] Canvas context is null");
        }
      } else {
        console.warn("[screenshot-overlay] canvasRef is null");
      }
    } catch (error) {
      console.error("[screenshot-overlay] Error processing screenshot", error);
    }
  }, [getAppWindow]);

  const handleScreenshotEvent = useCallback((event: any) => {
    // Handle different payload structures
    let base64: string;
    if (typeof event?.payload === 'string') {
      base64 = event.payload;
    } else if (event?.payload?.payload) {
      base64 = event.payload.payload;
    } else if (event?.payload?.data) {
      base64 = event.payload.data;
    } else {
      console.error("[screenshot-overlay] Invalid payload structure", event);
      return;
    }

    // Call the new handleScreenshotData function
    handleScreenshotData(base64);
  }, [handleScreenshotData]);

  useTauriListeners("open_screenshot_overlay", handleScreenshotEvent);

  const handlePointerDown = useCallback((e: React.PointerEvent) => {
    const overlayEl = overlayRef.current;
    if (!overlayEl) return;
    // only start when clicking the overlay background (same behavior as before)
    if (e.target !== overlayEl) return;

    // capture the pointer so we keep receiving move/up events even if the pointer goes over other elements
    try {
      overlayEl.setPointerCapture(e.pointerId);
    } catch (err) {
      console.warn("[screenshot-overlay] setPointerCapture failed", err);
    }

    const rect = overlayEl.getBoundingClientRect();
    const startX = e.clientX - rect.left;
    const startY = e.clientY - rect.top;

    setIsSelecting(true);
    setSelection({
      startX,
      startY,
      endX: startX,
      endY: startY,
    });
  }, []);

  const handlePointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!isSelecting || !selection) return;

      const overlayEl = overlayRef.current!;
      const rect = overlayEl.getBoundingClientRect();
      const endX = e.clientX - rect.left;
      const endY = e.clientY - rect.top;

      setSelection((prev) =>
        prev
          ? {
            ...prev,
            endX,
            endY,
          }
          : null,
      );
    },
    [isSelecting, selection],
  );

  // Helper function for scaling selection coordinates
  function getScaledSelection(
    canvas: HTMLCanvasElement,
    selection: SelectionArea,
  ) {
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;

    return {
      x: Math.min(selection.startX, selection.endX) * scaleX,
      y: Math.min(selection.startY, selection.endY) * scaleY,
      width: Math.abs(selection.endX - selection.startX) * scaleX,
      height: Math.abs(selection.endY - selection.startY) * scaleY,
    };
  }

  // Define cropSelection before handlePointerUp so it can be referenced
  const cropSelection = useCallback(() => {
    if (!canvasRef.current || !selection) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Apply scaling to selection
    const { x, y, width, height } = getScaledSelection(canvas, selection);

    // Get image data
    if (width < 1 || height < 1) {
      console.warn("[screenshot-overlay] Invalid selection area");
      setSelection(null);
      return;
    }
    const imageData = ctx.getImageData(x, y, width, height);

    // Create new canvas for cropped image
    const cropCanvas = document.createElement("canvas");
    cropCanvas.width = width;
    cropCanvas.height = height;
    const cropCtx = cropCanvas.getContext("2d");
    if (!cropCtx) return;

    cropCtx.putImageData(imageData, 0, 0);

    // Export cropped image as base64 PNG
    const croppedDataUrl = cropCanvas.toDataURL("image/png");
    return croppedDataUrl;
  }, [selection]);

  const handlePointerUp = useCallback(
    async (e: React.PointerEvent) => {
      const overlayEl = overlayRef.current;
      if (!overlayEl) return;

      // release pointer capture
      try {
        overlayEl.releasePointerCapture((e as any).pointerId);
      } catch (err) {
        console.warn("[screenshot-overlay] releasePointerCapture failed", err);
      }

      if (isSelecting && selection) {
        setIsSelecting(false);
        const croppedDataUrl = cropSelection();

        // Session check is now done before screenshot is taken
        // If we reach here, a session must be active
        try {
          await invoke("receive_screenshot_data", {
            imageUrl: croppedDataUrl,
            folderId: null,
            courseId: null,
            setId: null,
          });
          onClose();
        } catch (err) {
          console.error("[screenshot-overlay] Failed to save screenshot:", err);
          alert("Error saving screenshot. Please try again.");
          onClose();
        }
      }
    },
    [isSelecting, selection, cropSelection, onClose],
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setIsSelecting(false);
        setSelection(null);
        onClose();
      }
    },
    [onClose],
  );

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);

  const getSelectionStyle = () => {
    if (!selection) return {};

    const left = Math.min(selection.startX, selection.endX);
    const top = Math.min(selection.startY, selection.endY);
    const width = Math.abs(selection.endX - selection.startX);
    const height = Math.abs(selection.endY - selection.startY);

    return {
      left: `${left}px`,
      top: `${top}px`,
      width: `${width}px`,
      height: `${height}px`,
    };
  };

  return (
    <div
      ref={overlayRef}
      className="fixed inset-0 z-50 cursor-crosshair"
      style={{ backgroundColor: "rgba(0, 0, 0, 0.3)" }}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
    >
      {/* Canvas background */}
      <canvas
        ref={canvasRef}
        className="absolute top-0 left-0 z-0"
        style={{ pointerEvents: "none", width: "100%", height: "100%" }}
      />

      {/* Close button */}
      <button
        onClick={onClose}
        className="absolute top-4 right-4 z-60 bg-white/90 hover:bg-white text-gray-800 p-2 rounded-full shadow-lg transition-colors"
      >
        <X size={20} />
      </button>

      {/* Instructions */}
      <div className="absolute top-4 left-1/2 transform -translate-x-1/2 z-60 bg-white/90 text-gray-800 px-4 py-2 rounded-lg shadow-lg select-none">
        <p className="text-sm font-medium">
          Click and drag to select an area • Press ESC to cancel
        </p>
      </div>

      {/* Selection area */}
      {selection && (
        <>
          {/* Clear selection area */}
          <div
            className="absolute border-2 border-blue-500 bg-transparent"
            style={{
              ...getSelectionStyle(),
              boxShadow: "0 0 0 9999px rgba(0, 0, 0, 0.3)",
            }}
          />

          {/* Selection border with resize handles */}
          <div
            className="absolute border-2 border-blue-500"
            style={getSelectionStyle()}
          >
            {/* Corner resize handles */}
            <div className="absolute -top-1 -left-1 w-3 h-3 bg-blue-500 cursor-nw-resize" />
            <div className="absolute -top-1 -right-1 w-3 h-3 bg-blue-500 cursor-ne-resize" />
            <div className="absolute -bottom-1 -left-1 w-3 h-3 bg-blue-500 cursor-sw-resize" />
            <div className="absolute -bottom-1 -right-1 w-3 h-3 bg-blue-500 cursor-se-resize" />

            {/* Edge resize handles */}
            <div className="absolute -top-1 left-1/2 transform -translate-x-1/2 w-3 h-3 bg-blue-500 cursor-n-resize" />
            <div className="absolute -bottom-1 left-1/2 transform -translate-x-1/2 w-3 h-3 bg-blue-500 cursor-s-resize" />
            <div className="absolute -left-1 top-1/2 transform -translate-y-1/2 w-3 h-3 bg-blue-500 cursor-w-resize" />
            <div className="absolute -right-1 top-1/2 transform -translate-y-1/2 w-3 h-3 bg-blue-500 cursor-e-resize" />
          </div>

          {/* Selection dimensions */}
          {selection && (
            <div
              className="absolute bg-blue-500 text-white px-2 py-1 rounded text-xs font-medium"
              style={{
                left: `${Math.min(selection.startX, selection.endX)}px`,
                top: `${Math.min(selection.startY, selection.endY) - 25}px`,
              }}
            >
              {Math.trunc(Math.abs(selection.endX - selection.startX))} ×{" "}
              {Math.trunc(Math.abs(selection.endY - selection.startY))}
            </div>
          )}
        </>
      )}
    </div>
  );
}
