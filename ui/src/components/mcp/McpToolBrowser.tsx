// MCP tool browser component
import { useEffect } from "react";
import { useMcpStore } from "../../stores/mcpStore";

export function McpToolBrowser() {
  const { tools, toolsLoading, loadTools } = useMcpStore();

  useEffect(() => {
    loadTools();
  }, [loadTools]);

  if (toolsLoading) {
    return (
      <div className="p-4 text-gray-500 text-sm">Loading tools...</div>
    );
  }

  if (tools.length === 0) {
    return (
      <div className="p-4 text-gray-500 text-sm text-center">
        No tools available. Enable an MCP server to see its tools.
      </div>
    );
  }

  // Group tools by server
  const toolsByServer = tools.reduce((acc, tool) => {
    if (!acc[tool.server_name]) {
      acc[tool.server_name] = [];
    }
    acc[tool.server_name].push(tool);
    return acc;
  }, {} as Record<string, typeof tools>);

  return (
    <div className="space-y-4">
      {Object.entries(toolsByServer).map(([serverName, serverTools]) => (
        <div key={serverName} className="border border-gray-200 rounded-lg overflow-hidden">
          <div className="px-3 py-2 bg-gray-50 border-b border-gray-200">
            <h4 className="text-sm font-medium text-gray-700">{serverName}</h4>
            <p className="text-xs text-gray-500">{serverTools.length} tools</p>
          </div>
          <div className="divide-y divide-gray-100">
            {serverTools.map((tool) => (
              <div key={`${tool.server_id}-${tool.name}`} className="px-3 py-2">
                <div className="flex items-center gap-2">
                  <span className="font-mono text-sm text-gray-900">{tool.name}</span>
                </div>
                {tool.description && (
                  <p className="text-xs text-gray-500 mt-1">{tool.description}</p>
                )}
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
