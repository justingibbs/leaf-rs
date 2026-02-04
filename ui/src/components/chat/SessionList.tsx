// Session list sidebar component
import { useChatStore } from "../../stores/chatStore";
import type { ChatSession } from "../../types";

interface SessionListProps {
  onNewSession: () => void;
}

export function SessionList({ onNewSession }: SessionListProps) {
  const { sessions, activeSessionId, selectSession, sessionsLoading } = useChatStore();

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-700">
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">Chats</h3>
        <button
          onClick={onNewSession}
          className="p-1 text-gray-500 hover:text-leaf-600 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
          title="New Chat (Cmd+N)"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
        </button>
      </div>

      {/* Session list */}
      <div className="flex-1 overflow-y-auto">
        {sessionsLoading && sessions.length === 0 ? (
          <div className="p-3 text-sm text-gray-500 dark:text-gray-400">Loading...</div>
        ) : sessions.length === 0 ? (
          <div className="p-3 text-sm text-gray-500 dark:text-gray-400 text-center">
            No conversations yet.
            <br />
            <button
              onClick={onNewSession}
              className="text-leaf-600 hover:underline mt-1"
            >
              Start a new chat
            </button>
          </div>
        ) : (
          <div className="py-1">
            {sessions.map((session) => (
              <SessionItem
                key={session.id}
                session={session}
                isActive={session.id === activeSessionId}
                onSelect={() => selectSession(session.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface SessionItemProps {
  session: ChatSession;
  isActive: boolean;
  onSelect: () => void;
}

function SessionItem({ session, isActive, onSelect }: SessionItemProps) {
  const { archiveSession } = useChatStore();

  const handleArchive = (e: React.MouseEvent) => {
    e.stopPropagation();
    archiveSession(session.id);
  };

  // Format the date
  const date = new Date(session.updated_at);
  const isToday = date.toDateString() === new Date().toDateString();
  const timeStr = isToday
    ? date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    : date.toLocaleDateString([], { month: "short", day: "numeric" });

  return (
    <button
      onClick={onSelect}
      className={`w-full px-3 py-2 text-left transition-colors group ${
        isActive
          ? "bg-leaf-50 dark:bg-leaf-900/30 border-l-2 border-leaf-500"
          : "hover:bg-gray-50 dark:hover:bg-gray-800 border-l-2 border-transparent"
      }`}
    >
      <div className="flex items-center justify-between">
        <span
          className={`text-sm truncate ${
            isActive
              ? "text-leaf-700 dark:text-leaf-300 font-medium"
              : "text-gray-700 dark:text-gray-300"
          }`}
        >
          {session.title || "New Chat"}
        </span>
        <div className="flex items-center gap-1">
          <span className="text-xs text-gray-400 dark:text-gray-500">{timeStr}</span>
          <button
            onClick={handleArchive}
            className="p-0.5 text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
            title="Archive"
          >
            <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>
    </button>
  );
}
