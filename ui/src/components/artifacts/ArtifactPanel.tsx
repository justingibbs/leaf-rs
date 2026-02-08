// Artifact panel listing tracked artifacts with filters
import { useState } from "react";
import { useArtifacts, useScanArtifacts } from "../../hooks/useArtifacts";
import { ArtifactItem } from "./ArtifactItem";
import type { Artifact, ArtifactType, ArtifactStatus } from "../../types";

type TypeFilter = "all" | ArtifactType;
type CreatorFilter = "all" | "agent" | "card" | "user";
type StatusFilter = "all" | ArtifactStatus;

export function ArtifactPanel() {
  const { data: artifacts, isLoading } = useArtifacts();
  const scanMutation = useScanArtifacts();

  const [typeFilter, setTypeFilter] = useState<TypeFilter>("all");
  const [creatorFilter, setCreatorFilter] = useState<CreatorFilter>("all");
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

  const filtered = (artifacts ?? []).filter((a: Artifact) => {
    if (typeFilter !== "all" && a.artifact_type !== typeFilter) return false;
    if (creatorFilter !== "all" && a.created_by !== creatorFilter) return false;
    if (statusFilter !== "all" && a.status !== statusFilter) return false;
    return true;
  });

  return (
    <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Artifacts
          </h2>
          <button
            onClick={() => scanMutation.mutate()}
            disabled={scanMutation.isPending}
            className="px-2.5 py-1 text-xs font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors disabled:opacity-50"
            title="Scan project directory for new files"
          >
            {scanMutation.isPending ? "Scanning..." : "Scan"}
          </button>
        </div>

        {/* Filters */}
        <div className="flex items-center gap-2 mt-2 flex-wrap">
          {/* Type filter */}
          <select
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value as TypeFilter)}
            className="text-xs px-2 py-1 rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300"
          >
            <option value="all">All Types</option>
            <option value="file">Files</option>
            <option value="directory">Folders</option>
          </select>

          {/* Creator filter */}
          <select
            value={creatorFilter}
            onChange={(e) => setCreatorFilter(e.target.value as CreatorFilter)}
            className="text-xs px-2 py-1 rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300"
          >
            <option value="all">All Creators</option>
            <option value="agent">Agent</option>
            <option value="card">Card</option>
            <option value="user">User</option>
          </select>

          {/* Status filter */}
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
            className="text-xs px-2 py-1 rounded border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300"
          >
            <option value="all">All Status</option>
            <option value="active">Active</option>
            <option value="modified">Modified</option>
            <option value="deleted">Deleted</option>
          </select>
        </div>
      </div>

      {/* Artifact list */}
      <div className="divide-y divide-gray-100 dark:divide-gray-800 max-h-96 overflow-y-auto">
        {isLoading ? (
          <div className="p-4 text-center text-gray-500 dark:text-gray-400">
            <div className="animate-pulse">Loading artifacts...</div>
          </div>
        ) : filtered.length === 0 ? (
          <div className="p-8 text-center text-gray-500 dark:text-gray-400">
            <svg
              className="mx-auto h-12 w-12 text-gray-400 dark:text-gray-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"
              />
            </svg>
            <p className="mt-2 text-sm">No artifacts found</p>
            <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
              Click &ldquo;Scan&rdquo; to discover files in your project
            </p>
          </div>
        ) : (
          filtered.map((artifact: Artifact) => (
            <ArtifactItem key={artifact.id} artifact={artifact} />
          ))
        )}
      </div>

      {/* Footer count */}
      {filtered.length > 0 && (
        <div className="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
          <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
            {filtered.length} artifact{filtered.length !== 1 ? "s" : ""}
            {typeFilter !== "all" || creatorFilter !== "all" || statusFilter !== "all"
              ? " (filtered)"
              : ""}
          </p>
        </div>
      )}
    </div>
  );
}
