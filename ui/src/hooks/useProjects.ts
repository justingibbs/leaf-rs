import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useRecentProjects() {
  return useQuery({
    queryKey: ["recentProjects"],
    queryFn: api.listRecentProjects,
    staleTime: 1000 * 60, // 1 minute
  });
}

export function useCurrentProject() {
  return useQuery({
    queryKey: ["currentProject"],
    queryFn: api.getCurrentProject,
    staleTime: 1000 * 30, // 30 seconds
  });
}
