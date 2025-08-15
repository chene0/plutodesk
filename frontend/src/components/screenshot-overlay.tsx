"use client"

import type React from "react"

import { useState, useRef, useCallback, useEffect } from "react"
import { X, Copy, Download, Crop } from "lucide-react"

interface SelectionArea {
    startX: number
    startY: number
    endX: number
    endY: number
}

interface ScreenshotOverlayProps {
    onClose: () => void
}

export function ScreenshotOverlay({ onClose }: ScreenshotOverlayProps) {
    const [isSelecting, setIsSelecting] = useState(false)
    const [selection, setSelection] = useState<SelectionArea | null>(null)
    const [showToolbar, setShowToolbar] = useState(false)
    const overlayRef = useRef<HTMLDivElement>(null)

    const handleMouseDown = useCallback((e: React.MouseEvent) => {
        if (e.target !== overlayRef.current) return

        const rect = overlayRef.current!.getBoundingClientRect()
        const startX = e.clientX - rect.left
        const startY = e.clientY - rect.top

        setIsSelecting(true)
        setSelection({
            startX,
            startY,
            endX: startX,
            endY: startY,
        })
        setShowToolbar(false)
    }, [])

    const handleMouseMove = useCallback(
        (e: React.MouseEvent) => {
            if (!isSelecting || !selection) return

            const rect = overlayRef.current!.getBoundingClientRect()
            const endX = e.clientX - rect.left
            const endY = e.clientY - rect.top

            setSelection((prev) =>
                prev
                    ? {
                        ...prev,
                        endX,
                        endY,
                    }
                    : null,
            )
        },
        [isSelecting, selection],
    )

    const handleMouseUp = useCallback(() => {
        if (isSelecting && selection) {
            setIsSelecting(false)
            // Only show toolbar if there's a meaningful selection
            const width = Math.abs(selection.endX - selection.startX)
            const height = Math.abs(selection.endY - selection.startY)
            if (width > 10 && height > 10) {
                setShowToolbar(true)
            }
        }
    }, [isSelecting, selection])

    const handleKeyDown = useCallback(
        (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                onClose()
            }
        },
        [onClose],
    )

    useEffect(() => {
        document.addEventListener("keydown", handleKeyDown)
        return () => document.removeEventListener("keydown", handleKeyDown)
    }, [handleKeyDown])

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

    const getToolbarPosition = () => {
        if (!selection) return {}

        const left = Math.min(selection.startX, selection.endX)
        const top = Math.min(selection.startY, selection.endY)
        const width = Math.abs(selection.endX - selection.startX)
        const height = Math.abs(selection.endY - selection.startY)

        // Position toolbar below the selection, or above if near bottom
        const toolbarTop = top + height + 10
        const toolbarLeft = left + width / 2

        return {
            left: `${toolbarLeft}px`,
            top: `${toolbarTop}px`,
            transform: "translateX(-50%)",
        }
    }

    return (
        <div
            ref={overlayRef}
            className="fixed inset-0 z-50 cursor-crosshair"
            style={{ backgroundColor: "rgba(0, 0, 0, 0.3)" }}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
        >
            {/* Close button */}
            <button
                onClick={onClose}
                className="absolute top-4 right-4 z-60 bg-white/90 hover:bg-white text-gray-800 p-2 rounded-full shadow-lg transition-colors"
            >
                <X size={20} />
            </button>

            {/* Instructions */}
            <div className="absolute top-4 left-1/2 transform -translate-x-1/2 z-60 bg-white/90 text-gray-800 px-4 py-2 rounded-lg shadow-lg">
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
                            {Math.abs(selection.endX - selection.startX)} × {Math.abs(selection.endY - selection.startY)}
                        </div>
                    )}
                </>
            )}

            {/* Action toolbar */}
            {showToolbar && selection && (
                <div
                    className="absolute z-60 bg-white rounded-lg shadow-lg border p-2 flex items-center gap-2"
                    style={getToolbarPosition()}
                >
                    <button className="flex items-center gap-2 px-3 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors text-sm font-medium">
                        <Crop size={16} />
                        Capture
                    </button>
                    <button className="flex items-center gap-2 px-3 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors text-sm">
                        <Copy size={16} />
                        Copy
                    </button>
                    <button className="flex items-center gap-2 px-3 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors text-sm">
                        <Download size={16} />
                        Save
                    </button>
                    <div className="w-px h-6 bg-gray-300" />
                    <button onClick={onClose} className="p-2 text-gray-500 hover:text-gray-700 transition-colors">
                        <X size={16} />
                    </button>
                </div>
            )}
        </div>
    )
}
