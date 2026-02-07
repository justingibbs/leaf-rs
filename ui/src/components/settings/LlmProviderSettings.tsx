// LLM Provider settings component
import { useState, useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";

const LLM_PROVIDERS = [
  { value: "anthropic", label: "Anthropic", models: ["claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022", "claude-3-opus-20240229"] },
  { value: "openai", label: "OpenAI", models: ["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"] },
  { value: "google", label: "Google", models: ["gemini-2.0-flash", "gemini-2.5-pro", "gemini-2.5-flash"] },
  { value: "ollama", label: "Ollama", models: ["llama3.1", "codellama", "mistral"] },
];

export function LlmProviderSettings() {
  const { projectConfig, updateProjectSettings, projectConfigLoading, projectConfigError } = useSettingsStore();

  // Local form state
  const [provider, setProvider] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState(4096);
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [initialized, setInitialized] = useState(false);

  // Initialize form from projectConfig (only once on load)
  useEffect(() => {
    if (projectConfig?.llmSettings && !initialized) {
      const llm = projectConfig.llmSettings;
      setProvider(llm.provider);
      setModel(llm.model);
      setApiKey(llm.api_key || "");
      setBaseUrl(llm.base_url || "");
      setTemperature(llm.temperature);
      setMaxTokens(llm.max_tokens);
      setInitialized(true);
    }
  }, [projectConfig, initialized]);

  const selectedProvider = LLM_PROVIDERS.find(p => p.value === provider);
  const availableModels = selectedProvider?.models || [];

  const handleProviderChange = (newProvider: string) => {
    setProvider(newProvider);
    // Set a default model for the new provider
    const providerConfig = LLM_PROVIDERS.find(p => p.value === newProvider);
    if (providerConfig && providerConfig.models.length > 0) {
      setModel(providerConfig.models[0]);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      // Use camelCase to match Rust's serde rename_all = "camelCase"
      await updateProjectSettings({
        llmSettings: {
          provider,
          model,
          apiKey: apiKey || null,
          baseUrl: baseUrl || null,
          temperature,
          maxTokens: maxTokens,
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

      {/* Provider Selection */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          LLM Provider
        </label>
        <div className="grid grid-cols-2 gap-2">
          {LLM_PROVIDERS.map((p) => (
            <button
              key={p.value}
              type="button"
              onClick={() => handleProviderChange(p.value)}
              className={`px-4 py-3 text-sm font-medium rounded-lg border transition-colors ${
                provider === p.value
                  ? "bg-leaf-50 border-leaf-500 text-leaf-700"
                  : "bg-white border-gray-300 text-gray-700 hover:bg-gray-50"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      {/* Model Selection */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Model
        </label>
        <select
          value={model}
          onChange={(e) => setModel(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
        >
          {availableModels.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
          {!availableModels.includes(model) && model && (
            <option value={model}>{model} (custom)</option>
          )}
        </select>
        <p className="text-xs text-gray-500 mt-1">
          Or type a custom model name above
        </p>
      </div>

      {/* API Key */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          API Key
          {provider === "ollama" && (
            <span className="ml-2 text-xs text-gray-500">(optional for Ollama)</span>
          )}
        </label>
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder={provider === "ollama" ? "Not required for local Ollama" : "Enter your API key"}
          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
        />
        <p className="text-xs text-gray-500 mt-1">
          {provider === "anthropic" && "Get your API key from console.anthropic.com"}
          {provider === "openai" && "Get your API key from platform.openai.com"}
          {provider === "google" && "Get your API key from aistudio.google.com"}
          {provider === "ollama" && "Running locally - no API key needed"}
        </p>
      </div>

      {/* Base URL (for Ollama or custom endpoints) */}
      {(provider === "ollama" || baseUrl) && (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Base URL
          </label>
          <input
            type="text"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder={provider === "ollama" ? "http://localhost:11434" : "https://api.custom.com"}
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
          />
          <p className="text-xs text-gray-500 mt-1">
            Leave empty to use default endpoint
          </p>
        </div>
      )}

      {/* Advanced Settings */}
      <div className="pt-4 border-t border-gray-200">
        <h4 className="text-sm font-medium text-gray-700 mb-3">Advanced Settings</h4>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">
              Temperature ({temperature.toFixed(1)})
            </label>
            <input
              type="range"
              min="0"
              max="2"
              step="0.1"
              value={temperature}
              onChange={(e) => setTemperature(parseFloat(e.target.value))}
              className="w-full"
            />
            <p className="text-xs text-gray-500">
              Lower = more focused, Higher = more creative
            </p>
          </div>

          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">
              Max Tokens
            </label>
            <input
              type="number"
              value={maxTokens}
              onChange={(e) => setMaxTokens(parseInt(e.target.value) || 4096)}
              min={256}
              max={32768}
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm"
            />
          </div>
        </div>
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
