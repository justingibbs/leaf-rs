// MCP server list component
import { useEffect, useState } from "react";
import { useMcpStore } from "../../stores/mcpStore";
import { McpServerItem } from "./McpServerItem";
import { McpServerForm } from "./McpServerForm";

export function McpServerList() {
  const { servers, serversLoading, serversError, loadServers } = useMcpStore();
  const [showAddForm, setShowAddForm] = useState(false);

  useEffect(() => {
    loadServers();
  }, [loadServers]);

  const handleAddComplete = () => {
    setShowAddForm(false);
  };

  if (serversLoading && servers.length === 0) {
    return (
      <div className="p-4 text-gray-500 text-sm">Loading MCP servers...</div>
    );
  }

  return (
    <div className="space-y-4">
      {serversError && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-md text-sm text-red-700">
          {serversError}
        </div>
      )}

      {/* Server list */}
      {servers.length === 0 && !showAddForm ? (
        <div className="text-center py-6">
          <div className="inline-flex items-center justify-center w-12 h-12 bg-gray-100 rounded-full mb-3">
            <svg className="w-6 h-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M12 5l7 7-7 7" />
            </svg>
          </div>
          <p className="text-gray-500 text-sm mb-3">No MCP servers configured</p>
          <button
            onClick={() => setShowAddForm(true)}
            className="text-sm text-leaf-600 hover:text-leaf-700"
          >
            Add your first server
          </button>
        </div>
      ) : (
        <div className="space-y-2">
          {servers.map((server) => (
            <McpServerItem key={server.id} server={server} />
          ))}
        </div>
      )}

      {/* Add form */}
      {showAddForm ? (
        <McpServerForm
          onComplete={handleAddComplete}
          onCancel={() => setShowAddForm(false)}
        />
      ) : servers.length > 0 && (
        <button
          onClick={() => setShowAddForm(true)}
          className="w-full py-2 text-sm text-gray-500 hover:text-gray-700 hover:bg-gray-50 rounded-md border border-dashed border-gray-300 transition-colors"
        >
          + Add Server
        </button>
      )}

      {/* Info box */}
      <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
        <h5 className="text-sm font-medium text-blue-800 mb-1">About MCP Servers</h5>
        <p className="text-xs text-blue-700">
          MCP (Model Context Protocol) servers extend the AI agent's capabilities
          by providing additional tools. Configure servers to give the agent access
          to external services like databases, APIs, or file systems.
        </p>
      </div>
    </div>
  );
}
