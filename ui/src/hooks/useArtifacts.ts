// TanStack Query hooks for artifact operations
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useArtifacts(status?: string) {
  return useQuery({
    queryKey: ["artifacts", status],
    queryFn: () => api.listArtifacts(status),
    staleTime: 1000 * 15,
  });
}

export function useArtifact(artifactId: string | null) {
  return useQuery({
    queryKey: ["artifact", artifactId],
    queryFn: () => api.getArtifact(artifactId!),
    enabled: !!artifactId,
    staleTime: 1000 * 15,
  });
}

export function useScanArtifacts() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.scanArtifacts(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["artifacts"] });
    },
  });
}
