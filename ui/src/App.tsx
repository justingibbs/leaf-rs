import { useProjectStore } from "./stores/projectStore";
import { ProjectPicker } from "./components/ProjectPicker";
import { EventQueue } from "./components/EventQueue";
import { useLeafEvents } from "./hooks/useLeafEvents";
import "./App.css";

function App() {
  const { project, closeProject } = useProjectStore();

  // Subscribe to LEAF events from Rust
  useLeafEvents();

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
        <div className="max-w-4xl mx-auto space-y-6">
          {/* Welcome card */}
          <div className="bg-white rounded-lg border border-gray-200 p-6">
            <h2 className="text-lg font-medium text-gray-900 mb-4">
              Welcome to LEAF
            </h2>
            <p className="text-gray-600">
              Your project is ready. Configure watch paths in your project
              settings to start monitoring for file changes.
            </p>
            <div className="mt-6 p-4 bg-gray-50 rounded-lg">
              <p className="text-sm text-gray-500">
                Project path:{" "}
                <code className="text-gray-700">{project.path}</code>
              </p>
            </div>
          </div>

          {/* Event Queue */}
          <EventQueue />
        </div>
      </main>
    </div>
  );
}

export default App;
