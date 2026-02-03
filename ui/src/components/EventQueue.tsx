// Event queue component showing real-time file events
import { useEffect } from "react";
import { useEventStore } from "../stores/eventStore";
import { EventItem } from "./EventItem";
import { WatcherControls } from "./WatcherControls";

export function EventQueue() {
  const { events, isLoadingEvents, loadEvents } = useEventStore();

  useEffect(() => {
    loadEvents(50);
  }, [loadEvents]);

  return (
    <div className="bg-white rounded-lg border border-gray-200">
      <div className="px-4 py-3 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-medium text-gray-900">Event Queue</h2>
          <WatcherControls />
        </div>
      </div>

      <div className="divide-y divide-gray-100 max-h-96 overflow-y-auto">
        {isLoadingEvents ? (
          <div className="p-4 text-center text-gray-500">
            <div className="animate-pulse">Loading events...</div>
          </div>
        ) : events.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
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
            <p className="text-xs text-gray-400 mt-1">
              Events will appear here when files are detected
            </p>
          </div>
        ) : (
          events.map((event) => <EventItem key={event.id} event={event} />)
        )}
      </div>

      {events.length > 0 && (
        <div className="px-4 py-2 border-t border-gray-200 bg-gray-50">
          <p className="text-xs text-gray-500 text-center">
            Showing {events.length} recent event{events.length !== 1 ? "s" : ""}
          </p>
        </div>
      )}
    </div>
  );
}
