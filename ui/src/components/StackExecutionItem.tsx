// Stack execution item showing grouped per-card progress in the event queue
import type { StackExecution, ExecutionStatus } from "../types";
import { useStackStore } from "../stores/stackStore";

interface StackExecutionItemProps {
  execution: StackExecution;
}

export function StackExecutionItem({ execution }: StackExecutionItemProps) {
  const stacks = useStackStore((s) => s.stacks);
  const stack = stacks.find((s) => s.id === execution.stack_id);

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "Just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  const formatDuration = (ms: number | null) => {
    if (ms === null) return null;
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${Math.floor(ms / 60000)}m ${Math.round((ms % 60000) / 1000)}s`;
  };

  const statusConfig: Record<
    ExecutionStatus,
    { color: string; bgColor: string; label: string }
  > = {
    pending: {
      color: "text-yellow-600 dark:text-yellow-400",
      bgColor: "bg-yellow-100 dark:bg-yellow-900/30",
      label: "Pending",
    },
    running: {
      color: "text-blue-600 dark:text-blue-400",
      bgColor: "bg-blue-100 dark:bg-blue-900/30",
      label: "Running",
    },
    success: {
      color: "text-green-600 dark:text-green-400",
      bgColor: "bg-green-100 dark:bg-green-900/30",
      label: "Success",
    },
    failed: {
      color: "text-red-600 dark:text-red-400",
      bgColor: "bg-red-100 dark:bg-red-900/30",
      label: "Failed",
    },
    timeout: {
      color: "text-orange-600 dark:text-orange-400",
      bgColor: "bg-orange-100 dark:bg-orange-900/30",
      label: "Timeout",
    },
    cancelled: {
      color: "text-gray-600 dark:text-gray-400",
      bgColor: "bg-gray-100 dark:bg-gray-700",
      label: "Cancelled",
    },
  };

  const config = statusConfig[execution.status];
  const isRunning = execution.status === "running" || execution.status === "pending";
  const progress =
    execution.card_count > 0
      ? Math.round((execution.completed_cards / execution.card_count) * 100)
      : 0;

  return (
    <div className="px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
      <div className="flex items-start gap-3">
        {/* Stack execution icon */}
        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-leaf-100 dark:bg-leaf-900/30 flex items-center justify-center">
          {isRunning ? (
            <div className="w-4 h-4 border-2 border-leaf-500 border-t-transparent rounded-full animate-spin" />
          ) : (
            <svg
              className={`w-4 h-4 ${config.color}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
          )}
        </div>

        {/* Execution details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-900 dark:text-gray-100 truncate">
              {stack?.name ?? "Stack Execution"}
            </span>
            <span
              className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${config.bgColor} ${config.color}`}
            >
              {config.label}
            </span>
          </div>

          {/* Progress bar for running executions */}
          {isRunning && execution.card_count > 0 && (
            <div className="mt-1.5 flex items-center gap-2">
              <div className="flex-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                <div
                  className="h-full bg-leaf-500 rounded-full transition-all"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <span className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                {execution.completed_cards}/{execution.card_count}
              </span>
            </div>
          )}

          {/* Completed execution summary */}
          {!isRunning && (
            <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              {execution.completed_cards}/{execution.card_count} cards completed
              {execution.failed_at_position !== null && (
                <span className="text-red-500 dark:text-red-400">
                  {" "}
                  (failed at step {execution.failed_at_position + 1})
                </span>
              )}
            </div>
          )}

          {/* Error message */}
          {execution.error && (
            <p className="mt-1 text-xs text-red-500 dark:text-red-400 truncate">
              {execution.error}
            </p>
          )}

          {/* Metadata row */}
          <div className="flex items-center gap-3 mt-1 text-xs text-gray-400 dark:text-gray-500">
            <span>{formatTime(execution.started_at)}</span>
            {formatDuration(execution.duration_ms) && (
              <span>{formatDuration(execution.duration_ms)}</span>
            )}
          </div>
        </div>

        {/* Status indicator */}
        <div className="flex-shrink-0">
          {execution.status === "success" && (
            <svg
              className="w-5 h-5 text-green-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
          )}
          {execution.status === "failed" && (
            <svg
              className="w-5 h-5 text-red-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          )}
          {execution.status === "timeout" && (
            <svg
              className="w-5 h-5 text-orange-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          )}
        </div>
      </div>
    </div>
  );
}
