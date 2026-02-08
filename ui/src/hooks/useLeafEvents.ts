// Hook for subscribing to LEAF events from the Rust backend
import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LeafEvent } from "../types";
import { useEventStore } from "../stores/eventStore";
import { useStackStore } from "../stores/stackStore";
import { useExecutionStore } from "../stores/executionStore";
import { useChatStore } from "../stores/chatStore";
import { useMcpStore } from "../stores/mcpStore";

/**
 * Hook that subscribes to LEAF events from the Rust backend
 * and updates the relevant stores.
 */
export function useLeafEvents() {
  const { addEvent, updateEventStatus } = useEventStore();
  const {
    addStack,
    updateStackInStore,
    removeStack,
    setStackEnabled,
    addCard,
    updateCardInStore,
    removeCard,
    reloadCardsForStack,
  } = useStackStore();
  const { addExecution, updateExecution, updateExecutionStatus } =
    useExecutionStore();
  const {
    handleSessionCreated,
    handleSessionUpdated,
    handleMessageReceived,
    handleAgentThinking,
    handleAgentToolCall,
    handleChatError,
  } = useChatStore();
  const {
    handleServerConnected,
    handleServerDisconnected,
    handleServerError,
  } = useMcpStore();

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
            if (leafEvent.payload.context.startsWith("chat:")) {
              const sessionId = leafEvent.payload.context.replace("chat:", "");
              handleChatError(sessionId, leafEvent.payload.message);
            }
            break;

          case "warning":
            console.warn(
              `Warning in ${leafEvent.payload.context}:`,
              leafEvent.payload.message
            );
            break;

          // Stack events
          case "stack_created":
            console.log("Stack created:", leafEvent.payload.name);
            addStack(leafEvent.payload);
            break;

          case "stack_updated":
            console.log("Stack updated:", leafEvent.payload.name);
            updateStackInStore(leafEvent.payload);
            break;

          case "stack_deleted":
            console.log("Stack deleted:", leafEvent.payload.stack_id);
            removeStack(leafEvent.payload.stack_id);
            break;

          case "stack_enabled":
            console.log("Stack enabled:", leafEvent.payload.stack_id);
            setStackEnabled(leafEvent.payload.stack_id, true);
            break;

          case "stack_disabled":
            console.log("Stack disabled:", leafEvent.payload.stack_id);
            setStackEnabled(leafEvent.payload.stack_id, false);
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
            removeCard(leafEvent.payload.card_id, leafEvent.payload.stack_id);
            break;

          case "card_reordered":
            console.log("Cards reordered in stack:", leafEvent.payload.stack_id);
            reloadCardsForStack(leafEvent.payload.stack_id);
            break;

          // Stack execution events
          case "stack_execution_started":
            console.log("Stack execution started:", leafEvent.payload.id);
            addExecution(leafEvent.payload);
            break;

          case "stack_execution_progress":
            console.log("Stack execution progress:", leafEvent.payload.stack_execution_id);
            updateExecution(leafEvent.payload.stack_execution_id, {
              completed_cards: leafEvent.payload.completed_cards,
              card_count: leafEvent.payload.card_count,
            });
            break;

          case "stack_execution_completed":
            console.log(
              "Stack execution completed:",
              leafEvent.payload.stack_execution_id,
              leafEvent.payload.status
            );
            updateExecutionStatus(
              leafEvent.payload.stack_execution_id,
              leafEvent.payload.status
            );
            break;

          // Card execution events
          case "card_execution_started":
            console.log("Card execution started:", leafEvent.payload.id);
            break;

          case "card_execution_completed":
            console.log(
              "Card execution completed:",
              leafEvent.payload.card_execution_id,
              leafEvent.payload.status
            );
            break;

          // Chat/Session events
          case "session_created":
            console.log("Session created:", leafEvent.payload.id);
            handleSessionCreated(leafEvent.payload);
            break;

          case "session_updated":
            console.log("Session updated:", leafEvent.payload.id);
            handleSessionUpdated(leafEvent.payload);
            break;

          case "message_received":
            console.log("Message received:", leafEvent.payload.id);
            handleMessageReceived(leafEvent.payload);
            break;

          case "agent_thinking":
            console.log("Agent thinking:", leafEvent.payload.session_id);
            handleAgentThinking(leafEvent.payload.session_id);
            break;

          case "agent_tool_call":
            console.log("Agent tool call:", leafEvent.payload.tool_name);
            handleAgentToolCall(
              leafEvent.payload.session_id,
              leafEvent.payload.tool_name,
              leafEvent.payload.arguments
            );
            break;

          // MCP events
          case "mcp_server_connected":
            console.log("MCP server connected:", leafEvent.payload.server_name);
            handleServerConnected(leafEvent.payload.server_id, leafEvent.payload.tool_count);
            break;

          case "mcp_server_disconnected":
            console.log("MCP server disconnected:", leafEvent.payload.server_name);
            handleServerDisconnected(leafEvent.payload.server_id);
            break;

          case "mcp_server_error":
            console.error("MCP server error:", leafEvent.payload.server_name, leafEvent.payload.error);
            handleServerError(leafEvent.payload.server_id, leafEvent.payload.error);
            break;

          case "mcp_tool_called":
            console.log("MCP tool called:", leafEvent.payload.tool_name);
            break;

          case "mcp_tool_result":
            console.log("MCP tool result:", leafEvent.payload.tool_name, leafEvent.payload.success);
            break;

          default:
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
    addStack,
    updateStackInStore,
    removeStack,
    setStackEnabled,
    addCard,
    updateCardInStore,
    removeCard,
    reloadCardsForStack,
    addExecution,
    updateExecution,
    updateExecutionStatus,
    handleSessionCreated,
    handleSessionUpdated,
    handleMessageReceived,
    handleAgentThinking,
    handleAgentToolCall,
    handleChatError,
    handleServerConnected,
    handleServerDisconnected,
    handleServerError,
  ]);
}
