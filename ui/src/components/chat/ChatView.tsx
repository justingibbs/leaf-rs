// Main chat view component - combines session list and conversation
import { useEffect } from "react";
import { useChatStore } from "../../stores/chatStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { SessionList } from "./SessionList";
import { Conversation } from "./Conversation";

export function ChatView() {
  const { loadSessions, createSession, setAgentConfig } = useChatStore();
  const { projectConfig, loadProjectSettings } = useSettingsStore();

  // Load sessions on mount
  useEffect(() => {
    loadSessions();
    loadProjectSettings();
  }, [loadSessions, loadProjectSettings]);

  // Sync agent config from project settings
  useEffect(() => {
    if (projectConfig?.llmSettings) {
      const llm = projectConfig.llmSettings;
      setAgentConfig({
        api_key: llm.api_key || "",
        provider: llm.provider as "anthropic" | "openai" | "google" | "ollama",
        model: llm.model,
      });
    }
  }, [projectConfig, setAgentConfig]);

  // Keyboard shortcut for new session (Cmd+N)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "n") {
        e.preventDefault();
        createSession();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [createSession]);

  const handleNewSession = async () => {
    await createSession();
  };

  return (
    <div className="flex h-full bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
      {/* Session list sidebar */}
      <div className="w-56 border-r border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 flex-shrink-0">
        <SessionList onNewSession={handleNewSession} />
      </div>

      {/* Main conversation area */}
      <div className="flex-1 flex flex-col min-w-0">
        <Conversation />
      </div>
    </div>
  );
}
