// MCP server item component
import { useState } from "react";
import { useMcpStore } from "../../stores/mcpStore";
import type { McpServer } from "../../types";

interface McpServerItemProps {
  server: McpServer;
}

export function McpServerItem({ server }: McpServerItemProps) {
  const { enableServer, disableServer, removeServer, testServer, connectionStatus } =
    useMcpStore();
  const [isTesting, setIsTesting] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const status = connectionStatus.get(server.id);

  const handleToggle = async () => {
    if (server.enabled) {
      await disableServer(server.id);
    } else {
      await enableServer(server.id);
    }
  };

  const handleTest = async () => {
    setIsTesting(true);
    try {
      await testServer(server.id);
    } finally {
      setIsTesting(false);
    }
  };

  const handleDelete = async () => {
    if (confirm(`Are you sure you want to remove "${server.name}"?`)) {
      setIsDeleting(true);
      await removeServer(server.id);
    }
  };

  return (
    <div className="p-3 bg-white border border-gray-200 rounded-lg">
      <div className="flex items-start justify-between">
        {/* Server info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-900">{server.name}</span>
            {/* Connection status indicator */}
            {status && (
              <span
                className={`w-2 h-2 rounded-full ${
                  status.connected ? "bg-green-500" : "bg-gray-300"
                }`}
                title={status.connected ? `Connected (${status.tool_count} tools)` : "Disconnected"}
              />
            )}
          </div>
          <div className="mt-1 text-xs text-gray-500 font-mono truncate">
            {server.command} {server.args.join(" ")}
          </div>
          {status?.connected && (
            <div className="mt-1 text-xs text-green-600">
              {status.tool_count} tools available
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 ml-2">
          {/* Test button */}
          <button
            onClick={handleTest}
            disabled={isTesting}
            className="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors disabled:opacity-50"
            title="Test connection"
          >
            {isTesting ? (
              <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
            ) : (
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            )}
          </button>

          {/* Enable/disable toggle */}
          <button
            onClick={handleToggle}
            className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
              server.enabled ? "bg-leaf-500" : "bg-gray-200"
            }`}
            title={server.enabled ? "Disable" : "Enable"}
          >
            <span
              className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white shadow transition-transform ${
                server.enabled ? "translate-x-4" : "translate-x-1"
              }`}
            />
          </button>

          {/* Delete button */}
          <button
            onClick={handleDelete}
            disabled={isDeleting}
            className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors disabled:opacity-50"
            title="Remove server"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        </div>
      </div>

      {/* Environment variables */}
      {Object.keys(server.env).length > 0 && (
        <div className="mt-2 pt-2 border-t border-gray-100">
          <div className="text-xs text-gray-500">
            Environment: {Object.keys(server.env).length} variable(s)
          </div>
        </div>
      )}
    </div>
  );
}
