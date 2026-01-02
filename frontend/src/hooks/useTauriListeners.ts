import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriListeners(eventName: string, callback: (event: any) => void) {
    const callbackRef = useRef(callback);
    
    // Keep callback ref in sync with latest callback
    useEffect(() => {
        callbackRef.current = callback;
    }, [callback]);

    useEffect(() => {
        let unlisten: (() => void) | undefined;

        (async () => {
            try {
                unlisten = await listen(eventName, (event) => {
                    callbackRef.current(event);
                });
            } catch (error) {
                console.error(`Failed to register listener for event "${eventName}":`, error);
            }
        })();

        return () => {
            if (unlisten) unlisten();
        };
    }, [eventName]); // Removed callback from deps, using ref instead to avoid infinite loops
}
