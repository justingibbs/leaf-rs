// Settings modal component
import { useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { LlmProviderSettings } from "./LlmProviderSettings";
import { AppearanceSettings } from "./AppearanceSettings";
import { ExecutionSettings } from "./ExecutionSettings";
import { McpServerList } from "../mcp";

const TABS = [
  { id: "llm", label: "LLM Provider", icon: "🤖" },
  { id: "execution", label: "Execution", icon: "⚡" },
  { id: "mcp", label: "MCP Servers", icon: "🔌" },
  { id: "appearance", label: "Appearance", icon: "🎨" },
] as const;

type TabId = (typeof TABS)[number]["id"];

export function SettingsModal() {
  const { isSettingsOpen, activeTab, setActiveTab, closeSettings } = useSettingsStore();

  // Close on escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isSettingsOpen) {
        closeSettings();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isSettingsOpen, closeSettings]);

  if (!isSettingsOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 transition-opacity"
        onClick={closeSettings}
      />

      {/* Modal */}
      <div className="flex min-h-full items-center justify-center p-4">
        <div className="relative w-full max-w-2xl bg-white rounded-xl shadow-2xl">
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">Settings</h2>
            <button
              onClick={closeSettings}
              className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Tab navigation */}
          <div className="flex border-b border-gray-200">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as TabId)}
                className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
                  activeTab === tab.id
                    ? "text-leaf-600 border-b-2 border-leaf-600"
                    : "text-gray-500 hover:text-gray-700 border-b-2 border-transparent"
                }`}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </div>

          {/* Tab content */}
          <div className="p-6 max-h-[60vh] overflow-y-auto">
            {activeTab === "llm" && <LlmProviderSettings />}
            {activeTab === "execution" && <ExecutionSettings />}
            {activeTab === "mcp" && <McpServerList />}
            {activeTab === "appearance" && <AppearanceSettings />}
          </div>

          {/* Footer with keyboard hint */}
          <div className="px-6 py-3 bg-gray-50 rounded-b-xl border-t border-gray-200">
            <p className="text-xs text-gray-500 text-center">
              Press <kbd className="px-1.5 py-0.5 bg-gray-200 rounded text-gray-600 font-mono text-xs">Esc</kbd> to close
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
