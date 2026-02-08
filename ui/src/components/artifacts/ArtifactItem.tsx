// Individual artifact display
import type { Artifact } from "../../types";

interface ArtifactItemProps {
  artifact: Artifact;
}

export function ArtifactItem({ artifact }: ArtifactItemProps) {
  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "Just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  const formatSize = (bytes: number | null) => {
    if (bytes === null) return null;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const creatorColors: Record<string, string> = {
    agent: "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300",
    card: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
    user: "bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300",
  };

  const statusIndicator: Record<string, string> = {
    active: "bg-green-400",
    modified: "bg-yellow-400",
    deleted: "bg-red-400",
  };

  const isDeleted = artifact.status === "deleted";

  return (
    <div
      className={`px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors ${
        isDeleted ? "opacity-50" : ""
      }`}
    >
      <div className="flex items-center gap-2.5">
        {/* Type icon */}
        <div className="flex-shrink-0">
          {artifact.artifact_type === "directory" ? (
            <svg
              className="w-4 h-4 text-yellow-500"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
            </svg>
          ) : (
            <svg
              className="w-4 h-4 text-gray-400 dark:text-gray-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
              />
            </svg>
          )}
        </div>

        {/* Details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5">
            <span className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
              {artifact.filename}
            </span>
            <span
              className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${statusIndicator[artifact.status]}`}
              title={artifact.status}
            />
          </div>
          <div className="flex items-center gap-2 mt-0.5 text-xs text-gray-400 dark:text-gray-500">
            <span className="truncate" title={artifact.path}>
              {artifact.path}
            </span>
          </div>
        </div>

        {/* Right side: size, creator, time */}
        <div className="flex-shrink-0 flex items-center gap-2">
          {formatSize(artifact.size_bytes) && (
            <span className="text-xs text-gray-400 dark:text-gray-500">
              {formatSize(artifact.size_bytes)}
            </span>
          )}
          <span
            className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium ${
              creatorColors[artifact.created_by] ?? creatorColors.user
            }`}
          >
            {artifact.created_by}
          </span>
          <span className="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">
            {formatTime(artifact.modified_at)}
          </span>
        </div>
      </div>
    </div>
  );
}
