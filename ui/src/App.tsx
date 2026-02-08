import { useState, useEffect } from "react";
import { useProjectStore } from "./stores/projectStore";
import { useSettingsStore } from "./stores/settingsStore";
import { ProjectPicker } from "./components/ProjectPicker";
import { EventQueue } from "./components/EventQueue";
import { StackList, StackDetail } from "./components/stacks";
import { ChatView } from "./components/chat";
import { SettingsModal } from "./components/settings";
import { ToastContainer, ErrorBoundaryWrapper } from "./components/ui";
import { useLeafEvents } from "./hooks/useLeafEvents";
import { useTheme } from "./hooks/useTheme";
import type { Stack } from "./types";
import "./App.css";

type ActiveView = "chat" | "stacks";

function App() {
  const { project, closeProject } = useProjectStore();
  const { openSettings } = useSettingsStore();
  const [activeView, setActiveView] = useState<ActiveView>("chat");
  const [showStackDetail, setShowStackDetail] = useState(false);
  const [editingStack, setEditingStack] = useState<Stack | undefined>(undefined);

  // Subscribe to LEAF events from Rust
  useLeafEvents();

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        switch (e.key) {
          case ",":
            e.preventDefault();
            openSettings();
            break;
          case "1":
            e.preventDefault();
            setActiveView("chat");
            break;
          case "2":
            e.preventDefault();
            setActiveView("stacks");
            break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [openSettings]);

  const handleCreateStack = () => {
    setEditingStack(undefined);
    setShowStackDetail(true);
  };

  const handleSelectStack = (stack: Stack) => {
    setEditingStack(stack);
    setShowStackDetail(true);
  };

  const handleStackSaved = () => {
    setShowStackDetail(false);
    setEditingStack(undefined);
  };

  const handleCancelEdit = () => {
    setShowStackDetail(false);
    setEditingStack(undefined);
  };

  if (!project) {
    return <ProjectPicker />;
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-950">
      {/* Header */}
      <header className="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 px-4 py-3 flex-shrink-0">
        <div className="flex items-center justify-between">
          {/* Left: Logo and project name */}
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-leaf-500 rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">L</span>
            </div>
            <h1 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {project.name}
            </h1>
          </div>

          {/* Center: Navigation tabs */}
          <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-800 rounded-lg p-1">
            <button
              onClick={() => setActiveView("chat")}
              className={`px-4 py-1.5 text-sm font-medium rounded-md transition-colors ${
                activeView === "chat"
                  ? "bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm"
                  : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
              }`}
            >
              Chat
            </button>
            <button
              onClick={() => setActiveView("stacks")}
              className={`px-4 py-1.5 text-sm font-medium rounded-md transition-colors ${
                activeView === "stacks"
                  ? "bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm"
                  : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100"
              }`}
            >
              Stacks
            </button>
          </div>

          {/* Right: Settings and close */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => openSettings()}
              className="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
              title="Settings (Cmd+,)"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
            <button
              onClick={closeProject}
              className="px-3 py-1.5 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      </header>

      {/* Main content area - 3 column layout */}
      <main className="flex-1 flex overflow-hidden p-4 gap-4">
        {/* Left/Center column: Chat or Stacks */}
        <div className="flex-1 min-w-0">
          {activeView === "chat" ? (
            <ChatView />
          ) : showStackDetail ? (
            <div className="h-full bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 overflow-auto">
              <StackDetail
                stack={editingStack}
                onSave={handleStackSaved}
                onCancel={handleCancelEdit}
              />
            </div>
          ) : (
            <div className="h-full bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700 overflow-auto">
              <StackList
                onSelectStack={handleSelectStack}
                onCreateStack={handleCreateStack}
              />
            </div>
          )}
        </div>

        {/* Right column: Events & Executions */}
        <div className="w-96 flex-shrink-0">
          <div className="h-full overflow-auto">
            <EventQueue />
          </div>
        </div>
      </main>

      {/* Footer with keyboard hints */}
      <footer className="bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-800 px-4 py-2 flex-shrink-0">
        <div className="flex items-center justify-between text-xs text-gray-400 dark:text-gray-500">
          <div className="flex items-center gap-4">
            <span>
              <kbd className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">Cmd+1</kbd> Chat
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">Cmd+2</kbd> Stacks
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">Cmd+N</kbd> New Chat
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">Cmd+,</kbd> Settings
            </span>
          </div>
          <div className="flex items-center gap-2">
            <code className="text-gray-500 dark:text-gray-400">{project.path}</code>
          </div>
        </div>
      </footer>
    </div>
  );
}

// Settings modal rendered at the root level
function AppWithSettings() {
  // Apply theme based on settings
  useTheme();

  return (
    <ErrorBoundaryWrapper name="App">
      <App />
      <SettingsModal />
      <ToastContainer />
    </ErrorBoundaryWrapper>
  );
}

export default AppWithSettings;
