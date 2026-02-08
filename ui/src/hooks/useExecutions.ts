// TanStack Query hooks for execution operations
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/tauri";

export function useStackExecutions(limit?: number) {
  return useQuery({
    queryKey: ["executions", limit],
    queryFn: () => api.listExecutions(limit),
    staleTime: 1000 * 15,
  });
}

export function useStackExecution(executionId: string | null) {
  return useQuery({
    queryKey: ["execution", executionId],
    queryFn: () => api.getExecution(executionId!),
    enabled: !!executionId,
    staleTime: 1000 * 10,
  });
}

export function useExecutionsForEvent(eventId: string | null) {
  return useQuery({
    queryKey: ["executionsForEvent", eventId],
    queryFn: () => api.listExecutionsForEvent(eventId!),
    enabled: !!eventId,
    staleTime: 1000 * 15,
  });
}
