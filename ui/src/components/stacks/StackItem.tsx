// Individual stack item in the stack list
import type { Stack, TriggerConfig } from "../../types";
import { useStackStore } from "../../stores/stackStore";

interface StackItemProps {
  stack: Stack;
  cardCount: number;
  onSelect?: (stack: Stack) => void;
}

export function StackItem({ stack, cardCount, onSelect }: StackItemProps) {
  const { enableStack, disableStack, triggerStack, deleteStack } =
    useStackStore();

  const getTriggerIcon = (trigger: TriggerConfig): string => {
    switch (trigger.type) {
      case "file_created":
        return "M12 4v16m8-8H4";
      case "file_modified":
        return "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z";
      case "schedule":
        return "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z";
      case "manual":
        return "M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122";
    }
  };

  const getTriggerLabel = (trigger: TriggerConfig): string => {
    switch (trigger.type) {
      case "file_created":
        return `New files in ${trigger.watch_path}`;
      case "file_modified":
        return `Modified files in ${trigger.watch_path}`;
      case "schedule":
        return `Schedule: ${trigger.cron}`;
      case "manual":
        return "Manual trigger";
    }
  };

  const getPatterns = (trigger: TriggerConfig): string[] => {
    if (trigger.type === "file_created" || trigger.type === "file_modified") {
      return trigger.patterns;
    }
    return [];
  };

  const handleToggleEnabled = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      if (stack.enabled) {
        await disableStack(stack.id);
      } else {
        await enableStack(stack.id);
      }
    } catch (error) {
      console.error("Failed to toggle stack:", error);
    }
  };

  const handleTrigger = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await triggerStack(stack.id);
    } catch (error) {
      console.error("Failed to trigger stack:", error);
    }
  };

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(`Delete stack "${stack.name}" and all its cards?`)) {
      try {
        await deleteStack(stack.id);
      } catch (error) {
        console.error("Failed to delete stack:", error);
      }
    }
  };

  const patterns = getPatterns(stack.trigger);

  return (
    <div
      className={`px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer ${
        !stack.enabled ? "opacity-60" : ""
      }`}
      onClick={() => onSelect?.(stack)}
    >
      <div className="flex items-start gap-3">
        {/* Trigger type icon */}
        <div
          className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${
            stack.enabled
              ? "bg-leaf-100 dark:bg-leaf-900/30"
              : "bg-gray-100 dark:bg-gray-800"
          }`}
        >
          <svg
            className={`w-5 h-5 ${
              stack.enabled
                ? "text-leaf-600 dark:text-leaf-400"
                : "text-gray-400 dark:text-gray-500"
            }`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d={getTriggerIcon(stack.trigger)}
            />
          </svg>
        </div>

        {/* Stack details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-900 dark:text-gray-100 truncate">
              {stack.name}
            </span>
            {!stack.enabled && (
              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">
                Disabled
              </span>
            )}
            {cardCount > 1 && (
              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                {cardCount} cards
              </span>
            )}
          </div>

          {stack.description && (
            <p className="text-sm text-gray-500 dark:text-gray-400 truncate mt-0.5">
              {stack.description}
            </p>
          )}

          <div className="flex items-center gap-3 mt-1 text-xs text-gray-400 dark:text-gray-500">
            <span>{getTriggerLabel(stack.trigger)}</span>
            {patterns.length > 0 && (
              <span className="text-leaf-600 dark:text-leaf-400">
                {patterns.join(", ")}
              </span>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex-shrink-0 flex items-center gap-1">
          <button
            onClick={handleToggleEnabled}
            className={`p-1.5 rounded transition-colors ${
              stack.enabled
                ? "text-leaf-600 hover:bg-leaf-100 dark:text-leaf-400 dark:hover:bg-leaf-900/30"
                : "text-gray-400 hover:bg-gray-100 dark:text-gray-500 dark:hover:bg-gray-800"
            }`}
            title={stack.enabled ? "Disable stack" : "Enable stack"}
          >
            <svg
              className="w-5 h-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              {stack.enabled ? (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              ) : (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              )}
            </svg>
          </button>

          {stack.enabled && cardCount > 0 && (
            <button
              onClick={handleTrigger}
              className="p-1.5 rounded text-gray-400 hover:text-leaf-600 hover:bg-leaf-100 dark:text-gray-500 dark:hover:text-leaf-400 dark:hover:bg-leaf-900/30 transition-colors"
              title="Run now"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </button>
          )}

          <button
            onClick={handleDelete}
            className="p-1.5 rounded text-gray-400 hover:text-red-600 hover:bg-red-50 dark:text-gray-500 dark:hover:text-red-400 dark:hover:bg-red-900/20 transition-colors"
            title="Delete stack"
          >
            <svg
              className="w-5 h-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
