// Individual stack execution item
import { useState } from "react";
import type { StackExecution, ExecutionStatus } from "../../types";

interface ExecutionItemProps {
  execution: StackExecution;
}

export function ExecutionItem({ execution }: ExecutionItemProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const getStatusIcon = (status: ExecutionStatus): JSX.Element => {
    switch (status) {
      case "pending":
        return (
          <svg
            className="w-5 h-5 text-gray-400 animate-pulse"
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
        );
      case "running":
        return (
          <svg
            className="w-5 h-5 text-blue-500 animate-spin"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        );
      case "success":
        return (
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
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        );
      case "failed":
        return (
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
              d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        );
      case "timeout":
        return (
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
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            />
          </svg>
        );
      case "cancelled":
        return (
          <svg
            className="w-5 h-5 text-gray-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636"
            />
          </svg>
        );
    }
  };

  const getStatusLabel = (status: ExecutionStatus): string => {
    switch (status) {
      case "pending":
        return "Pending";
      case "running":
        return "Running";
      case "success":
        return "Success";
      case "failed":
        return "Failed";
      case "timeout":
        return "Timeout";
      case "cancelled":
        return "Cancelled";
    }
  };

  const getStatusColor = (status: ExecutionStatus): string => {
    switch (status) {
      case "pending":
        return "bg-gray-100 text-gray-600";
      case "running":
        return "bg-blue-100 text-blue-600";
      case "success":
        return "bg-green-100 text-green-600";
      case "failed":
        return "bg-red-100 text-red-600";
      case "timeout":
        return "bg-orange-100 text-orange-600";
      case "cancelled":
        return "bg-gray-100 text-gray-600";
    }
  };

  const formatDuration = (ms: number | null): string => {
    if (ms === null) return "-";
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${Math.floor(ms / 60000)}m ${((ms % 60000) / 1000).toFixed(0)}s`;
  };

  const formatTime = (isoString: string): string => {
    const date = new Date(isoString);
    return date.toLocaleTimeString();
  };

  const hasError = !!execution.error;

  return (
    <div className="border-b border-gray-100 dark:border-gray-800 last:border-b-0">
      <div
        className={`px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors ${
          hasError ? "cursor-pointer" : ""
        }`}
        onClick={() => hasError && setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center gap-3">
          <div className="flex-shrink-0">{getStatusIcon(execution.status)}</div>

          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-medium text-gray-900 dark:text-gray-100 truncate">
                Stack Execution
              </span>
              <span
                className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getStatusColor(
                  execution.status
                )}`}
              >
                {getStatusLabel(execution.status)}
              </span>
              {execution.card_count > 1 && (
                <span className="text-xs text-gray-400 dark:text-gray-500">
                  {execution.completed_cards}/{execution.card_count} cards
                </span>
              )}
            </div>

            <div className="flex items-center gap-3 mt-1 text-xs text-gray-400 dark:text-gray-500">
              <span>Started {formatTime(execution.started_at)}</span>
              {execution.duration_ms !== null && (
                <span>Duration: {formatDuration(execution.duration_ms)}</span>
              )}
            </div>
          </div>

          {hasError && (
            <div className="flex-shrink-0">
              <svg
                className={`w-5 h-5 text-gray-400 transition-transform ${
                  isExpanded ? "rotate-180" : ""
                }`}
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </div>
          )}
        </div>
      </div>

      {isExpanded && hasError && (
        <div className="px-4 pb-3">
          <div className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">
            Error:
          </div>
          <pre className="bg-gray-900 text-red-300 p-3 rounded text-xs overflow-x-auto max-h-48 overflow-y-auto">
            {execution.error}
          </pre>
        </div>
      )}
    </div>
  );
}
