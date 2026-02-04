// List of executions
import { useEffect } from "react";
import { useExecutionStore, getRunningExecutions } from "../../stores/executionStore";
import { ExecutionItem } from "./ExecutionItem";

interface ExecutionListProps {
  cardId?: string;
  limit?: number;
}

export function ExecutionList({ cardId, limit = 20 }: ExecutionListProps) {
  const { executions, isLoading, error, loadExecutions } = useExecutionStore();

  useEffect(() => {
    loadExecutions(cardId, limit);
  }, [cardId, limit, loadExecutions]);

  if (isLoading) {
    return (
      <div className="p-4 text-center text-gray-500">
        <div className="animate-pulse">Loading executions...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-center text-red-500">
        <p>Error loading executions: {error}</p>
      </div>
    );
  }

  if (executions.length === 0) {
    return (
      <div className="p-4 text-center text-gray-500">
        <p>No executions yet</p>
        <p className="text-sm mt-1">
          Trigger a card or add files to a watched folder to see executions
        </p>
      </div>
    );
  }

  // Sort executions by started_at (most recent first)
  const sortedExecutions = [...executions].sort(
    (a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime()
  );

  return (
    <div className="divide-y divide-gray-100">
      {sortedExecutions.map((execution) => (
        <ExecutionItem key={execution.id} execution={execution} />
      ))}
    </div>
  );
}

// Compact execution indicator for the sidebar or header
export function ExecutionIndicator() {
  const running = getRunningExecutions();

  if (running.length === 0) {
    return null;
  }

  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-blue-50 text-blue-600 text-sm rounded">
      <svg
        className="w-4 h-4 animate-spin"
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
      <span>
        {running.length} execution{running.length !== 1 ? "s" : ""} running
      </span>
    </div>
  );
}
