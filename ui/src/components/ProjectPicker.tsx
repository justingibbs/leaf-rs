import { useState } from "react";
import { useProjectStore } from "../stores/projectStore";
import { useRecentProjects } from "../hooks/useProjects";

export function ProjectPicker() {
  const [path, setPath] = useState("");
  const { openProject, createProject, loading, error } = useProjectStore();
  const { data: recentProjects } = useRecentProjects();

  const handleOpen = async () => {
    if (path.trim()) {
      await openProject(path.trim());
    }
  };

  const handleCreate = async () => {
    if (path.trim()) {
      await createProject(path.trim());
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center p-4">
      <div className="bg-white rounded-xl shadow-lg p-8 w-full max-w-md">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-12 h-12 bg-leaf-500 rounded-xl flex items-center justify-center">
            <span className="text-white font-bold text-xl">L</span>
          </div>
          <div>
            <h1 className="text-2xl font-bold text-gray-900">LEAF</h1>
            <p className="text-sm text-gray-500">Local Event-Driven Automation</p>
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <label
              htmlFor="path"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              Project Path
            </label>
            <input
              id="path"
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="/path/to/project"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-leaf-500 focus:border-transparent outline-none"
              disabled={loading}
            />
          </div>

          {error && (
            <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-600">{error}</p>
            </div>
          )}

          <div className="flex gap-3">
            <button
              onClick={handleOpen}
              disabled={loading || !path.trim()}
              className="flex-1 px-4 py-2 bg-leaf-500 text-white rounded-lg hover:bg-leaf-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loading ? "Opening..." : "Open Project"}
            </button>
            <button
              onClick={handleCreate}
              disabled={loading || !path.trim()}
              className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loading ? "Creating..." : "Create New"}
            </button>
          </div>
        </div>

        {recentProjects && recentProjects.length > 0 && (
          <div className="mt-8">
            <h2 className="text-sm font-medium text-gray-500 mb-3">
              Recent Projects
            </h2>
            <div className="space-y-2">
              {recentProjects.map((project) => (
                <button
                  key={project.path}
                  onClick={() => openProject(project.path)}
                  disabled={loading}
                  className="w-full text-left p-3 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
                >
                  <p className="font-medium text-gray-900">{project.name}</p>
                  <p className="text-sm text-gray-500 truncate">{project.path}</p>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
