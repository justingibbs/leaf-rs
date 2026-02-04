// MCP server form component
import { useState } from "react";
import { useMcpStore } from "../../stores/mcpStore";

interface McpServerFormProps {
  onComplete: () => void;
  onCancel: () => void;
}

export function McpServerForm({ onComplete, onCancel }: McpServerFormProps) {
  const { addServer } = useMcpStore();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [name, setName] = useState("");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [envVars, setEnvVars] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError("Server name is required");
      return;
    }

    if (!command.trim()) {
      setError("Command is required");
      return;
    }

    setIsSubmitting(true);

    try {
      // Parse args
      const argsList = args
        .split(/\s+/)
        .map((a) => a.trim())
        .filter(Boolean);

      // Parse env vars (format: KEY=value, one per line)
      const envObj: Record<string, string> = {};
      if (envVars.trim()) {
        for (const line of envVars.split("\n")) {
          const trimmed = line.trim();
          if (!trimmed || trimmed.startsWith("#")) continue;
          const [key, ...valueParts] = trimmed.split("=");
          if (key && valueParts.length > 0) {
            envObj[key.trim()] = valueParts.join("=").trim();
          }
        }
      }

      await addServer({
        name: name.trim(),
        command: command.trim(),
        args: argsList,
        env: envObj,
      });

      onComplete();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
      <h4 className="text-sm font-medium text-gray-900 mb-3">Add MCP Server</h4>

      {error && (
        <div className="mb-3 p-2 bg-red-50 border border-red-200 rounded text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Name */}
      <div className="mb-3">
        <label className="block text-xs font-medium text-gray-700 mb-1">
          Name <span className="text-red-500">*</span>
        </label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g., File System Server"
          className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
        />
      </div>

      {/* Command */}
      <div className="mb-3">
        <label className="block text-xs font-medium text-gray-700 mb-1">
          Command <span className="text-red-500">*</span>
        </label>
        <input
          type="text"
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          placeholder="e.g., npx or /path/to/server"
          className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 font-mono"
        />
      </div>

      {/* Arguments */}
      <div className="mb-3">
        <label className="block text-xs font-medium text-gray-700 mb-1">
          Arguments
        </label>
        <input
          type="text"
          value={args}
          onChange={(e) => setArgs(e.target.value)}
          placeholder="e.g., -y @modelcontextprotocol/server-filesystem /path"
          className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 font-mono"
        />
        <p className="text-xs text-gray-500 mt-1">Space-separated arguments</p>
      </div>

      {/* Environment Variables */}
      <div className="mb-4">
        <label className="block text-xs font-medium text-gray-700 mb-1">
          Environment Variables
        </label>
        <textarea
          value={envVars}
          onChange={(e) => setEnvVars(e.target.value)}
          placeholder="KEY=value&#10;ANOTHER_KEY=another_value"
          rows={3}
          className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 font-mono"
        />
        <p className="text-xs text-gray-500 mt-1">One per line, format: KEY=value</p>
      </div>

      {/* Actions */}
      <div className="flex justify-end gap-2">
        <button
          type="button"
          onClick={onCancel}
          className="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800 transition-colors"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={isSubmitting}
          className="px-4 py-1.5 text-sm font-medium text-white bg-leaf-600 rounded-md hover:bg-leaf-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSubmitting ? "Adding..." : "Add Server"}
        </button>
      </div>
    </form>
  );
}
