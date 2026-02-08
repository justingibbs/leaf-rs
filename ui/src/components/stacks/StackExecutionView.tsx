// Stack execution view showing grouped execution results
import type { StackExecution, ExecutionStatus } from "../../types";

interface StackExecutionViewProps {
  executions: StackExecution[];
}

export function StackExecutionView({ executions }: StackExecutionViewProps) {
  if (executions.length === 0) {
    return (
      <div className="p-4 text-center text-gray-500 dark:text-gray-400 text-sm">
        No executions yet
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-100 dark:divide-gray-800">
      {executions.map((exec) => (
        <ExecutionRow key={exec.id} execution={exec} />
      ))}
    </div>
  );
}

function ExecutionRow({ execution }: { execution: StackExecution }) {
  const statusConfig = getStatusConfig(execution.status);
  const startedAt = new Date(execution.started_at);
  const timeAgo = formatTimeAgo(startedAt);

  return (
    <div className="px-4 py-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className={`inline-flex items-center gap-1.5 ${statusConfig.className}`}>
            {statusConfig.icon}
            <span className="text-xs font-medium">{statusConfig.label}</span>
          </span>
          <span className="text-xs text-gray-400 dark:text-gray-500">
            {timeAgo}
          </span>
        </div>

        <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
          {execution.card_count > 0 && (
            <span>
              {execution.completed_cards}/{execution.card_count} cards
            </span>
          )}
          {execution.duration_ms !== null && (
            <span>{formatDuration(execution.duration_ms)}</span>
          )}
        </div>
      </div>

      {/* Progress bar for multi-card stacks */}
      {execution.card_count > 1 && (
        <div className="mt-2 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
          <div
            className={`h-1.5 rounded-full transition-all ${
              execution.status === "failed"
                ? "bg-red-500"
                : execution.status === "success"
                  ? "bg-green-500"
                  : "bg-leaf-500"
            }`}
            style={{
              width: `${execution.card_count > 0 ? (execution.completed_cards / execution.card_count) * 100 : 0}%`,
            }}
          />
        </div>
      )}

      {execution.error && (
        <p className="mt-1 text-xs text-red-500 dark:text-red-400 truncate">
          {execution.error}
        </p>
      )}
    </div>
  );
}

function getStatusConfig(status: ExecutionStatus) {
  switch (status) {
    case "pending":
      return {
        label: "Pending",
        className: "text-gray-500 dark:text-gray-400",
        icon: (
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        ),
      };
    case "running":
      return {
        label: "Running",
        className: "text-blue-600 dark:text-blue-400",
        icon: (
          <svg className="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
        ),
      };
    case "success":
      return {
        label: "Success",
        className: "text-green-600 dark:text-green-400",
        icon: (
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
        ),
      };
    case "failed":
      return {
        label: "Failed",
        className: "text-red-600 dark:text-red-400",
        icon: (
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        ),
      };
    case "timeout":
      return {
        label: "Timeout",
        className: "text-orange-600 dark:text-orange-400",
        icon: (
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        ),
      };
    case "cancelled":
      return {
        label: "Cancelled",
        className: "text-gray-500 dark:text-gray-400",
        icon: (
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
          </svg>
        ),
      };
  }
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

function formatTimeAgo(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const seconds = Math.floor(diff / 1000);

  if (seconds < 60) return "just now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return date.toLocaleDateString();
}
