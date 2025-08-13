import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriListeners(eventName: string, callback: (event: any) => void) {
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        (async () => {
            unlisten = await listen(eventName, (event) => {
                callback(event);
            });
        })();

        return () => {
            if (unlisten) unlisten();
        };
    }, []);
}
