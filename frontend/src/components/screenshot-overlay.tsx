"use client"

import type React from "react"

import { useState, useRef, useCallback, useEffect } from "react"
import { X, Copy, Download, Crop } from "lucide-react"
import { useTauriListeners } from "@/hooks/useTauriListeners"
import { Window } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event"

interface SelectionArea {
    startX: number
    startY: number
    endX: number
    endY: number
}

interface ScreenshotOverlayProps {
    onClose: () => void
}

const appWindow = new Window('screenshot_overlay');

export function ScreenshotOverlay({ onClose }: ScreenshotOverlayProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const [isSelecting, setIsSelecting] = useState(false)
    const [selection, setSelection] = useState<SelectionArea | null>(null)
    const overlayRef = useRef<HTMLDivElement>(null)

    useEffect(() => {
        console.log('[screenshot-overlay] emit ready');
        emit("screenshot_overlay_ready", { label: "screenshot_overlay" });
    }, []);

    useTauriListeners("open_screenshot_overlay", (event: any) => {
        console.log('[screenshot-overlay] received open_screenshot_overlay event', {
            id: event?.id,
            payloadLength: event?.payload?.length ?? 0,
        });

        const base64 = event.payload;
        const img = new window.Image();
        img.src = `data:image/png;base64,${base64}`;
        img.onload = async () => {
            console.log('[screenshot-overlay] image loaded', { width: img.width, height: img.height });
            const canvas = canvasRef.current;
            if (canvas) {
                canvas.width = img.width;
                canvas.height = img.height;
                console.log('[screenshot-overlay] canvas resized', { canvasWidth: canvas.width, canvasHeight: canvas.height });
                const ctx = canvas.getContext("2d");
                ctx?.drawImage(img, 0, 0);
                console.log('[screenshot-overlay] image drawn to canvas');
                await appWindow.show();
                console.log('[screenshot-overlay] requested window show');
            } else {
                console.warn('[screenshot-overlay] canvasRef is null when image loaded');
            }
        };
    });

    const handlePointerDown = useCallback((e: React.PointerEvent) => {
        const overlayEl = overlayRef.current;
        if (!overlayEl) return;
        // only start when clicking the overlay background (same behavior as before)
        if (e.target !== overlayEl) return;

        // capture the pointer so we keep receiving move/up events even if the pointer goes over other elements
        try {
            overlayEl.setPointerCapture(e.pointerId);
            console.log('[screenshot-overlay] setPointerCapture', e.pointerId);
        } catch (err) {
            console.warn('[screenshot-overlay] setPointerCapture failed', err);
        }

        const rect = overlayEl.getBoundingClientRect();
        const startX = e.clientX - rect.left;
        const startY = e.clientY - rect.top;

        console.log('[screenshot-overlay] pointerdown', { pointerId: e.pointerId, clientX: e.clientX, clientY: e.clientY, rect });

        setIsSelecting(true);
        setSelection({
            startX,
            startY,
            endX: startX,
            endY: startY,
        });
        console.log('[screenshot-overlay] selection started', { startX, startY });
    }, []);

    const handlePointerMove = useCallback((e: React.PointerEvent) => {
        if (!isSelecting || !selection) return;

        const overlayEl = overlayRef.current!;
        const rect = overlayEl.getBoundingClientRect();
        const endX = e.clientX - rect.left;
        const endY = e.clientY - rect.top;

        console.log('[screenshot-overlay] pointermove while selecting', {
            pointerId: (e as any).pointerId,
            clientX: e.clientX,
            clientY: e.clientY,
            rect,
            computedEnd: { endX, endY },
            prevSelection: selection,
        });

        setSelection((prev) =>
            prev
                ? {
                    ...prev,
                    endX,
                    endY,
                }
                : null,
        );
    }, [isSelecting, selection]);

    const handlePointerUp = useCallback((e: React.PointerEvent) => {
        const overlayEl = overlayRef.current;
        if (!overlayEl) return;

        // release pointer capture
        try {
            overlayEl.releasePointerCapture((e as any).pointerId);
            console.log('[screenshot-overlay] releasePointerCapture', (e as any).pointerId);
        } catch (err) {
            console.warn('[screenshot-overlay] releasePointerCapture failed', err);
        }

        console.log('[screenshot-overlay] pointerup', { isSelecting, selection, pointerId: (e as any).pointerId });
        if (isSelecting && selection) {
            setIsSelecting(false);
            const croppedDataUrl = cropSelection();
            console.log('[screenshot-overlay] cropSelection result length', croppedDataUrl?.length ?? 0);
        }
    }, [isSelecting, selection]);

    const handleKeyDown = useCallback(
        (e: KeyboardEvent) => {
            console.log('[screenshot-overlay] keydown', e.key);
            if (e.key === "Escape") {
                console.log('[screenshot-overlay] escape pressed -> closing overlay');
                setIsSelecting(false);
                setSelection(null);
                onClose()
            }
        },
        [onClose],
    )

    useEffect(() => {
        console.log('[screenshot-overlay] adding keydown listener');
        document.addEventListener("keydown", handleKeyDown)
        return () => {
            console.log('[screenshot-overlay] removing keydown listener');
            document.removeEventListener("keydown", handleKeyDown)
        }
    }, [handleKeyDown])

    // log selection changes to trace freezes
    useEffect(() => {
        console.log('[screenshot-overlay] selection changed', selection);
    }, [selection]);

    const getSelectionStyle = () => {
        if (!selection) return {}

        const left = Math.min(selection.startX, selection.endX)
        const top = Math.min(selection.startY, selection.endY)
        const width = Math.abs(selection.endX - selection.startX)
        const height = Math.abs(selection.endY - selection.startY)

        return {
            left: `${left}px`,
            top: `${top}px`,
            width: `${width}px`,
            height: `${height}px`,
        }
    }

    function getScaledSelection(canvas: HTMLCanvasElement, selection: SelectionArea) {
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;

        const result = {
            x: Math.min(selection.startX, selection.endX) * scaleX,
            y: Math.min(selection.startY, selection.endY) * scaleY,
            width: Math.abs(selection.endX - selection.startX) * scaleX,
            height: Math.abs(selection.endY - selection.startY) * scaleY,
        };
        console.log('[screenshot-overlay] getScaledSelection', { rect, scaleX, scaleY, result });
        return result;
    }

    function cropSelection() {
        if (!canvasRef.current || !selection) return;

        const canvas = canvasRef.current;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        console.log('[screenshot-overlay] cropSelection start', { selection, canvasWidth: canvas.width, canvasHeight: canvas.height });

        // Apply scaling to selection
        const { x, y, width, height } = getScaledSelection(canvas, selection);

        // Get image data
        console.log('[screenshot-overlay] getImageData params', { x, y, width, height });
        if (width < 1 || height < 1) {
            console.warn('[screenshot-overlay] Invalid selection area');
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
        console.log('[screenshot-overlay] cropSelection done', {
            croppedDataUrl: croppedDataUrl,
            croppedDataUrlLength: croppedDataUrl.length
        });
        return croppedDataUrl;
    }

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
                className="absolute top-0 left-0 w-full h-full z-0"
                style={{ pointerEvents: "none" }}
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
                <p className="text-sm font-medium">Click and drag to select an area • Press ESC to cancel</p>
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
                    <div className="absolute border-2 border-blue-500" style={getSelectionStyle()}>
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
                            {Math.trunc(Math.abs(selection.endX - selection.startX))} × {Math.trunc(Math.abs(selection.endY - selection.startY))}
                        </div>
                    )}
                </>
            )}
        </div>
    )
}
