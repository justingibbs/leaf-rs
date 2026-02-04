// Hook for subscribing to LEAF events from the Rust backend
import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LeafEvent } from "../types";
import { useEventStore } from "../stores/eventStore";
import { useCardStore } from "../stores/cardStore";
import { useExecutionStore } from "../stores/executionStore";

/**
 * Hook that subscribes to LEAF events from the Rust backend
 * and updates the relevant stores.
 */
export function useLeafEvents() {
  const { addEvent, updateEventStatus } = useEventStore();
  const { addCard, updateCardInStore, removeCard, loadCards } = useCardStore();
  const { addExecution, updateExecution, updateExecutionStatus } =
    useExecutionStore();

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

          // Card events
          case "card_created":
            console.log("Card created:", leafEvent.payload.name);
            addCard(leafEvent.payload);
            break;

          case "card_updated":
            console.log("Card updated:", leafEvent.payload.name);
            updateCardInStore(leafEvent.payload);
            break;

          case "card_deleted":
            console.log("Card deleted:", leafEvent.payload.card_id);
            removeCard(leafEvent.payload.card_id);
            break;

          case "card_enabled":
            console.log("Card enabled:", leafEvent.payload.card_id);
            // Reload cards to get updated state
            loadCards();
            break;

          case "card_disabled":
            console.log("Card disabled:", leafEvent.payload.card_id);
            // Reload cards to get updated state
            loadCards();
            break;

          // Execution events
          case "execution_started":
            console.log("Execution started:", leafEvent.payload.id);
            addExecution(leafEvent.payload);
            break;

          case "execution_progress":
            console.log("Execution progress:", leafEvent.payload.execution_id);
            updateExecution(leafEvent.payload.execution_id, {
              stdout: leafEvent.payload.stdout,
              stderr: leafEvent.payload.stderr,
            });
            break;

          case "execution_completed":
            console.log(
              "Execution completed:",
              leafEvent.payload.execution_id,
              leafEvent.payload.status
            );
            updateExecutionStatus(
              leafEvent.payload.execution_id,
              leafEvent.payload.status,
              leafEvent.payload.exit_code
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
  }, [
    addEvent,
    updateEventStatus,
    addCard,
    updateCardInStore,
    removeCard,
    loadCards,
    addExecution,
    updateExecution,
    updateExecutionStatus,
  ]);
}
