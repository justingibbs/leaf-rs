import { useState } from "react";
import { useProjectStore } from "./stores/projectStore";
import { ProjectPicker } from "./components/ProjectPicker";
import { EventQueue } from "./components/EventQueue";
import { CardList, CardEditor } from "./components/cards";
import { useLeafEvents } from "./hooks/useLeafEvents";
import type { Card } from "./types";
import "./App.css";

function App() {
  const { project, closeProject } = useProjectStore();
  const [showCardEditor, setShowCardEditor] = useState(false);
  const [editingCard, setEditingCard] = useState<Card | undefined>(undefined);

  // Subscribe to LEAF events from Rust
  useLeafEvents();

  const handleCreateCard = () => {
    setEditingCard(undefined);
    setShowCardEditor(true);
  };

  const handleSelectCard = (card: Card) => {
    setEditingCard(card);
    setShowCardEditor(true);
  };

  const handleCardSaved = () => {
    setShowCardEditor(false);
    setEditingCard(undefined);
  };

  const handleCancelEdit = () => {
    setShowCardEditor(false);
    setEditingCard(undefined);
  };

  if (!project) {
    return <ProjectPicker />;
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-leaf-500 rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">L</span>
            </div>
            <h1 className="text-xl font-semibold text-gray-900">
              {project.name}
            </h1>
          </div>
          <button
            onClick={closeProject}
            className="text-sm text-gray-500 hover:text-gray-700 transition-colors"
          >
            Close Project
          </button>
        </div>
      </header>
      <main className="p-6">
        <div className="max-w-6xl mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Left column: Cards */}
            <div className="space-y-6">
              {showCardEditor ? (
                <CardEditor
                  card={editingCard}
                  onSave={handleCardSaved}
                  onCancel={handleCancelEdit}
                />
              ) : (
                <CardList
                  onSelectCard={handleSelectCard}
                  onCreateCard={handleCreateCard}
                />
              )}

              {/* Project info */}
              <div className="bg-white rounded-lg border border-gray-200 p-4">
                <h3 className="text-sm font-medium text-gray-700 mb-2">
                  Project Info
                </h3>
                <p className="text-xs text-gray-500">
                  Path:{" "}
                  <code className="text-gray-700 bg-gray-100 px-1 rounded">
                    {project.path}
                  </code>
                </p>
              </div>
            </div>

            {/* Right column: Event Queue */}
            <div>
              <EventQueue />
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
