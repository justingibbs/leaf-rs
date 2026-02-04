// Execution settings component
import { useState, useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";

export function ExecutionSettings() {
  const { projectConfig, updateProjectSettings, projectConfigLoading, projectConfigError } = useSettingsStore();

  // Local form state
  const [timeoutSecs, setTimeoutSecs] = useState(300);
  const [maxRetries, setMaxRetries] = useState(3);
  const [allowNetwork, setAllowNetwork] = useState(false);
  const [denoPermissions, setDenoPermissions] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Initialize form from projectConfig
  useEffect(() => {
    if (projectConfig?.executionSettings) {
      const exec = projectConfig.executionSettings;
      setTimeoutSecs(exec.timeout_secs);
      setMaxRetries(exec.max_retries);
      setAllowNetwork(exec.allow_network);
      setDenoPermissions(exec.deno_permissions.join(", "));
    }
  }, [projectConfig]);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      await updateProjectSettings({
        executionSettings: {
          timeout_secs: timeoutSecs,
          max_retries: maxRetries,
          allow_network: allowNetwork,
          deno_permissions: denoPermissions
            .split(",")
            .map((p) => p.trim())
            .filter(Boolean),
        },
      });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } finally {
      setIsSaving(false);
    }
  };

  if (projectConfigLoading && !projectConfig) {
    return (
      <div className="p-4 text-gray-500 text-sm">Loading settings...</div>
    );
  }

  return (
    <div className="space-y-6">
      {projectConfigError && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-md text-sm text-red-700">
          {projectConfigError}
        </div>
      )}

      {/* Timeout */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Default Timeout (seconds)
        </label>
        <input
          type="number"
          value={timeoutSecs}
          onChange={(e) => setTimeoutSecs(parseInt(e.target.value) || 300)}
          min={10}
          max={3600}
          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
        />
        <p className="text-xs text-gray-500 mt-1">
          Maximum time a card can run before being terminated (10-3600 seconds)
        </p>
      </div>

      {/* Max Retries */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Default Max Retries
        </label>
        <input
          type="number"
          value={maxRetries}
          onChange={(e) => setMaxRetries(parseInt(e.target.value) || 0)}
          min={0}
          max={10}
          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
        />
        <p className="text-xs text-gray-500 mt-1">
          Number of times to retry a failed card execution (0-10)
        </p>
      </div>

      {/* Network Access */}
      <div className="pt-4 border-t border-gray-200">
        <h4 className="text-sm font-medium text-gray-700 mb-3">Sandbox Permissions</h4>

        <label className="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            checked={allowNetwork}
            onChange={(e) => setAllowNetwork(e.target.checked)}
            className="h-4 w-4 text-leaf-600 rounded border-gray-300 focus:ring-leaf-500"
          />
          <div>
            <span className="text-sm font-medium text-gray-700">Allow Network Access</span>
            <p className="text-xs text-gray-500">
              Enable cards to make HTTP requests and access the network
            </p>
          </div>
        </label>
      </div>

      {/* Additional Deno Permissions */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Additional Deno Permissions
        </label>
        <input
          type="text"
          value={denoPermissions}
          onChange={(e) => setDenoPermissions(e.target.value)}
          placeholder="e.g., --allow-env, --allow-ffi"
          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
        />
        <p className="text-xs text-gray-500 mt-1">
          Comma-separated list of additional Deno permission flags
        </p>
      </div>

      {/* Info Box */}
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h5 className="text-sm font-medium text-blue-800 mb-1">About the Sandbox</h5>
        <p className="text-xs text-blue-700">
          Cards run in a secure Deno sandbox with limited permissions. By default, they can only
          access files within the project folder. Enable additional permissions carefully.
        </p>
      </div>

      {/* Save Button */}
      <div className="flex justify-end pt-4">
        <button
          onClick={handleSave}
          disabled={isSaving}
          className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
            saveSuccess
              ? "bg-green-600 text-white"
              : "bg-leaf-600 text-white hover:bg-leaf-700"
          } disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          {isSaving ? "Saving..." : saveSuccess ? "Saved!" : "Save Settings"}
        </button>
      </div>
    </div>
  );
}
