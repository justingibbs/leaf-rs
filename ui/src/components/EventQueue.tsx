// Event queue component showing real-time file events and stack executions
import { useEffect } from "react";
import { useEventStore } from "../stores/eventStore";
import { useExecutionStore } from "../stores/executionStore";
import { EventItem } from "./EventItem";
import { StackExecutionItem } from "./StackExecutionItem";
import { WatcherControls } from "./WatcherControls";

export function EventQueue() {
  const { events, isLoadingEvents, loadEvents } = useEventStore();
  const { executions } = useExecutionStore();

  useEffect(() => {
    loadEvents(50);
  }, [loadEvents]);

  // Merge events and executions into a timeline sorted by time
  const timelineItems: Array<
    | { kind: "event"; data: (typeof events)[0]; time: number }
    | { kind: "execution"; data: (typeof executions)[0]; time: number }
  > = [];

  for (const event of events) {
    timelineItems.push({
      kind: "event",
      data: event,
      time: new Date(event.created_at).getTime(),
    });
  }

  for (const exec of executions) {
    timelineItems.push({
      kind: "execution",
      data: exec,
      time: new Date(exec.started_at).getTime(),
    });
  }

  timelineItems.sort((a, b) => b.time - a.time);

  return (
    <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Event Queue
          </h2>
          <WatcherControls />
        </div>
      </div>

      <div className="divide-y divide-gray-100 dark:divide-gray-800 max-h-96 overflow-y-auto">
        {isLoadingEvents ? (
          <div className="p-4 text-center text-gray-500 dark:text-gray-400">
            <div className="animate-pulse">Loading events...</div>
          </div>
        ) : timelineItems.length === 0 ? (
          <div className="p-8 text-center text-gray-500 dark:text-gray-400">
            <svg
              className="mx-auto h-12 w-12 text-gray-400 dark:text-gray-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
            <p className="mt-2 text-sm">No events yet</p>
            <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
              Events will appear here when files are detected
            </p>
          </div>
        ) : (
          timelineItems.map((item) =>
            item.kind === "event" ? (
              <EventItem key={`event-${item.data.id}`} event={item.data} />
            ) : (
              <StackExecutionItem
                key={`exec-${item.data.id}`}
                execution={item.data}
              />
            )
          )
        )}
      </div>

      {timelineItems.length > 0 && (
        <div className="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
            {events.length} event{events.length !== 1 ? "s" : ""}
            {executions.length > 0 &&
              `, ${executions.length} execution${executions.length !== 1 ? "s" : ""}`}
          </p>
        </div>
      )}
    </div>
  );
}
