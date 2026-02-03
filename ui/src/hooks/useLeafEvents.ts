// Hook for subscribing to LEAF events from the Rust backend
import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LeafEvent } from "../types";
import { useEventStore } from "../stores/eventStore";

/**
 * Hook that subscribes to LEAF events from the Rust backend
 * and updates the relevant stores.
 */
export function useLeafEvents() {
  const { addEvent, updateEventStatus } = useEventStore();

  useEffect(() => {
    let unlistenFn: UnlistenFn | null = null;

    const setupListener = async () => {
      unlistenFn = await listen<LeafEvent>("leaf-event", (event) => {
        const leafEvent = event.payload;

        console.log("Received LEAF event:", leafEvent.type, leafEvent.payload);

        switch (leafEvent.type) {
          case "event_created":
            addEvent(leafEvent.payload);
            break;

          case "event_completed":
            updateEventStatus(leafEvent.payload.event_id, leafEvent.payload.status);
            break;

          case "event_processing":
            updateEventStatus(leafEvent.payload.event_id, "processing");
            break;

          case "file_detected":
            // File was detected but not yet processed into an event
            console.log("File detected:", leafEvent.payload.path);
            break;

          case "watcher_started":
            console.log("Watcher started:", leafEvent.payload.paths);
            useEventStore.setState({
              isWatcherRunning: true,
              watchedPaths: leafEvent.payload.paths,
            });
            break;

          case "watcher_stopped":
            console.log("Watcher stopped");
            useEventStore.setState({
              isWatcherRunning: false,
              watchedPaths: [],
            });
            break;

          case "watcher_error":
            console.error("Watcher error:", leafEvent.payload.error);
            break;

          case "error":
            console.error(
              `Error in ${leafEvent.payload.context}:`,
              leafEvent.payload.message
            );
            break;

          case "warning":
            console.warn(
              `Warning in ${leafEvent.payload.context}:`,
              leafEvent.payload.message
            );
            break;

          default:
            // Handle other event types as needed
            console.log("Unhandled event type:", leafEvent);
        }
      });
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [addEvent, updateEventStatus]);
}
