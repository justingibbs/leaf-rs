// Watcher controls for starting/stopping file watching
import { useEffect } from "react";
import { useEventStore } from "../stores/eventStore";

export function WatcherControls() {
  const {
    isWatcherRunning,
    watchedPaths,
    startWatcher,
    stopWatcher,
    checkWatcherStatus,
  } = useEventStore();

  useEffect(() => {
    checkWatcherStatus();
  }, [checkWatcherStatus]);

  const handleToggle = async () => {
    if (isWatcherRunning) {
      await stopWatcher();
    } else {
      await startWatcher();
    }
  };

  return (
    <div className="flex items-center gap-3">
      {isWatcherRunning && watchedPaths.length > 0 && (
        <span className="text-xs text-gray-500">
          Watching {watchedPaths.length} path{watchedPaths.length !== 1 ? "s" : ""}
        </span>
      )}

      <button
        onClick={handleToggle}
        className={`
          inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium
          transition-colors
          ${
            isWatcherRunning
              ? "bg-red-50 text-red-700 hover:bg-red-100"
              : "bg-leaf-50 text-leaf-700 hover:bg-leaf-100"
          }
        `}
      >
        {isWatcherRunning ? (
          <>
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2 w-2 bg-red-500"></span>
            </span>
            Stop Watching
          </>
        ) : (
          <>
            <svg
              className="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
              />
            </svg>
            Start Watching
          </>
        )}
      </button>
    </div>
  );
}
