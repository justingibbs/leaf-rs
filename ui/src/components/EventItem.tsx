// Individual event item in the event queue
import type { Event, EventPayload } from "../types";

interface EventItemProps {
  event: Event;
}

export function EventItem({ event }: EventItemProps) {
  const statusColors: Record<Event["status"], string> = {
    pending: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300",
    processing: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
    completed: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
    failed: "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300",
  };

  const eventTypeIcons: Record<Event["event_type"], string> = {
    file_created: "M12 4v16m8-8H4",
    file_modified: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z",
    schedule: "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z",
    manual: "M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122",
  };

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "Just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  const getFileName = (payload: EventPayload) => {
    if (payload.type === "file") {
      const parts = payload.path.split("/");
      return parts[parts.length - 1];
    }
    return null;
  };

  const getFilePath = (payload: EventPayload) => {
    if (payload.type === "file") return payload.path;
    return null;
  };

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return null;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const fileName = getFileName(event.payload);
  const filePath = getFilePath(event.payload);
  const fileSize =
    event.payload.type === "file" ? formatFileSize(event.payload.size) : null;

  return (
    <div className="px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
      <div className="flex items-start gap-3">
        {/* Event type icon */}
        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center">
          <svg
            className="w-4 h-4 text-gray-600 dark:text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d={eventTypeIcons[event.event_type]}
            />
          </svg>
        </div>

        {/* Event details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-900 dark:text-gray-100 truncate">
              {fileName || event.event_type}
            </span>
            <span
              className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${statusColors[event.status]}`}
            >
              {event.status}
            </span>
          </div>

          {filePath && (
            <p
              className="text-sm text-gray-500 dark:text-gray-400 truncate mt-0.5"
              title={filePath}
            >
              {filePath}
            </p>
          )}

          <div className="flex items-center gap-3 mt-1 text-xs text-gray-400 dark:text-gray-500">
            <span>{formatTime(event.created_at)}</span>
            {fileSize && <span>{fileSize}</span>}
            {event.payload.type === "file" && event.payload.mime_type && (
              <span>{event.payload.mime_type}</span>
            )}
            {event.matched_stacks.length > 0 && (
              <span className="text-leaf-600 dark:text-leaf-400">
                {event.matched_stacks.length} stack
                {event.matched_stacks.length !== 1 ? "s" : ""} matched
              </span>
            )}
          </div>
        </div>

        {/* Status indicator */}
        <div className="flex-shrink-0">
          {event.status === "processing" && (
            <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          )}
          {event.status === "completed" && (
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
          {event.status === "failed" && (
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
        </div>
      </div>
    </div>
  );
}
