// Message bubble component
import type { Message, ToolCall } from "../../types";

interface MessageBubbleProps {
  message: Message;
}

export function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === "user";
  const isAssistant = message.role === "assistant";
  const isTool = message.role === "tool";

  return (
    <div
      className={`flex ${isUser ? "justify-end" : "justify-start"} mb-3`}
    >
      <div
        className={`max-w-[85%] rounded-lg px-4 py-2 ${
          isUser
            ? "bg-leaf-600 text-white"
            : isTool
            ? "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700"
            : "bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 border border-gray-200 dark:border-gray-700"
        }`}
      >
        {/* Role indicator for non-user messages */}
        {!isUser && (
          <div className="flex items-center gap-2 mb-1">
            <span
              className={`text-xs font-medium ${
                isAssistant
                  ? "text-leaf-600 dark:text-leaf-400"
                  : "text-gray-500 dark:text-gray-400"
              }`}
            >
              {isAssistant ? "Assistant" : "Tool Result"}
            </span>
          </div>
        )}

        {/* Message content */}
        <div className="text-sm whitespace-pre-wrap break-words">
          {message.content}
        </div>

        {/* Tool calls */}
        {message.tool_calls && message.tool_calls.length > 0 && (
          <div className="mt-2 space-y-2">
            {message.tool_calls.map((toolCall) => (
              <ToolCallDisplay key={toolCall.id} toolCall={toolCall} />
            ))}
          </div>
        )}

        {/* Timestamp */}
        <div
          className={`text-xs mt-1 ${
            isUser ? "text-leaf-200" : "text-gray-400 dark:text-gray-500"
          }`}
        >
          {formatTime(message.created_at)}
        </div>
      </div>
    </div>
  );
}

interface ToolCallDisplayProps {
  toolCall: ToolCall;
}

function ToolCallDisplay({ toolCall }: ToolCallDisplayProps) {
  const isCardTool = toolCall.name.includes("card");
  const result = toolCall.result as Record<string, unknown> | undefined;
  const cardName = result && typeof result === "object" && "name" in result ? String(result.name) : null;

  return (
    <div className="bg-gray-50 dark:bg-gray-800 rounded-md p-2 border border-gray-200 dark:border-gray-700">
      <div className="flex items-center gap-2 text-xs">
        <span className="px-1.5 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded font-mono">
          {toolCall.name}
        </span>
        {toolCall.result !== undefined && (
          <span className="text-green-600 dark:text-green-400">completed</span>
        )}
      </div>

      {/* Show card creation result */}
      {isCardTool && cardName && (
        <div className="mt-2 text-xs text-gray-600 dark:text-gray-400">
          <span>
            Created card: <strong>{cardName}</strong>
          </span>
        </div>
      )}
    </div>
  );
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

// Thinking indicator component
export function ThinkingIndicator() {
  return (
    <div className="flex justify-start mb-3">
      <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg px-4 py-2">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-leaf-600 dark:text-leaf-400">
            Assistant
          </span>
        </div>
        <div className="flex items-center gap-1 mt-1">
          <div className="w-2 h-2 bg-leaf-500 rounded-full animate-bounce" style={{ animationDelay: "0ms" }} />
          <div className="w-2 h-2 bg-leaf-500 rounded-full animate-bounce" style={{ animationDelay: "150ms" }} />
          <div className="w-2 h-2 bg-leaf-500 rounded-full animate-bounce" style={{ animationDelay: "300ms" }} />
        </div>
      </div>
    </div>
  );
}

// Tool call in progress indicator
interface ToolCallIndicatorProps {
  toolName: string;
}

export function ToolCallIndicator({ toolName }: ToolCallIndicatorProps) {
  return (
    <div className="flex justify-start mb-3">
      <div className="bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800 rounded-lg px-4 py-2">
        <div className="flex items-center gap-2 text-sm text-blue-700 dark:text-blue-300">
          <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          <span>Calling {toolName}...</span>
        </div>
      </div>
    </div>
  );
}
