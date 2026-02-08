// Stack list component showing all automation stacks
import { useEffect, useState } from "react";
import { useStackStore } from "../../stores/stackStore";
import { StackItem } from "./StackItem";
import type { Stack } from "../../types";

interface StackListProps {
  onSelectStack?: (stack: Stack) => void;
  onCreateStack?: () => void;
}

export function StackList({ onSelectStack, onCreateStack }: StackListProps) {
  const { stacks, cardsByStack, isLoading, error, loadStacks, loadCards } =
    useStackStore();
  const [filter, setFilter] = useState<"all" | "enabled" | "disabled">("all");

  useEffect(() => {
    loadStacks();
  }, [loadStacks]);

  // Load cards for each stack once stacks are loaded
  useEffect(() => {
    for (const stack of stacks) {
      if (!cardsByStack[stack.id]) {
        loadCards(stack.id);
      }
    }
  }, [stacks, cardsByStack, loadCards]);

  const filteredStacks = stacks.filter((stack) => {
    if (filter === "enabled") return stack.enabled;
    if (filter === "disabled") return !stack.enabled;
    return true;
  });

  const enabledCount = stacks.filter((s) => s.enabled).length;
  const disabledCount = stacks.filter((s) => !s.enabled).length;

  return (
    <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Stacks
          </h2>
          <button
            onClick={onCreateStack}
            className="inline-flex items-center px-3 py-1.5 text-sm font-medium rounded-md text-white bg-leaf-600 hover:bg-leaf-700 transition-colors"
          >
            <svg
              className="w-4 h-4 mr-1.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 4v16m8-8H4"
              />
            </svg>
            New Stack
          </button>
        </div>

        {stacks.length > 0 && (
          <div className="flex gap-2 mt-3">
            <button
              onClick={() => setFilter("all")}
              className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                filter === "all"
                  ? "bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900"
                  : "bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
              }`}
            >
              All ({stacks.length})
            </button>
            <button
              onClick={() => setFilter("enabled")}
              className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                filter === "enabled"
                  ? "bg-leaf-600 text-white"
                  : "bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
              }`}
            >
              Enabled ({enabledCount})
            </button>
            <button
              onClick={() => setFilter("disabled")}
              className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                filter === "disabled"
                  ? "bg-gray-600 text-white"
                  : "bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
              }`}
            >
              Disabled ({disabledCount})
            </button>
          </div>
        )}
      </div>

      <div className="divide-y divide-gray-100 dark:divide-gray-800 max-h-[calc(100vh-250px)] overflow-y-auto">
        {isLoading ? (
          <div className="p-4 text-center text-gray-500 dark:text-gray-400">
            <div className="animate-pulse">Loading stacks...</div>
          </div>
        ) : error ? (
          <div className="p-4 text-center text-red-500">
            <p className="text-sm">Failed to load stacks</p>
            <p className="text-xs mt-1">{error}</p>
          </div>
        ) : filteredStacks.length === 0 ? (
          <div className="p-8 text-center text-gray-500 dark:text-gray-400">
            <svg
              className="mx-auto h-12 w-12 text-gray-400 dark:text-gray-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
            <p className="mt-2 text-sm">
              {filter === "all" ? "No stacks yet" : `No ${filter} stacks`}
            </p>
            {filter === "all" && (
              <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                Create a stack to automate tasks when files are added
              </p>
            )}
          </div>
        ) : (
          filteredStacks.map((stack) => (
            <StackItem
              key={stack.id}
              stack={stack}
              cardCount={(cardsByStack[stack.id] ?? []).length}
              onSelect={onSelectStack}
            />
          ))
        )}
      </div>

      {filteredStacks.length > 0 && (
        <div className="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
            Showing {filteredStacks.length} of {stacks.length} stack
            {stacks.length !== 1 ? "s" : ""}
          </p>
        </div>
      )}
    </div>
  );
}
