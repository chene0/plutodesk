import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriListeners(eventName: string, callback: (event: any) => void) {
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        (async () => {
            try {
                unlisten = await listen(eventName, (event) => {
                    callback(event);
                });
            } catch (error) {
                console.error(`Failed to register listener for event "${eventName}":`, error);
            }
        })();

        return () => {
            if (unlisten) unlisten();
        };
    }, []);
}
