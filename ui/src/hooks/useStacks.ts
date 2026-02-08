// TanStack Query hooks for stack operations
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/tauri";
import type { CreateStackInput, UpdateStackInput } from "../types";

export function useStacks() {
  return useQuery({
    queryKey: ["stacks"],
    queryFn: api.listStacks,
    staleTime: 1000 * 30,
  });
}

export function useStack(stackId: string | null) {
  return useQuery({
    queryKey: ["stack", stackId],
    queryFn: () => api.getStack(stackId!),
    enabled: !!stackId,
    staleTime: 1000 * 30,
  });
}

export function useStackCards(stackId: string | null) {
  return useQuery({
    queryKey: ["stackCards", stackId],
    queryFn: () => api.listCards(stackId!),
    enabled: !!stackId,
    staleTime: 1000 * 15,
  });
}

export function useCreateStack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateStackInput) => api.createStack(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stacks"] });
    },
  });
}

export function useUpdateStack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      stackId,
      input,
    }: {
      stackId: string;
      input: UpdateStackInput;
    }) => api.updateStack(stackId, input),
    onSuccess: (_data, { stackId }) => {
      queryClient.invalidateQueries({ queryKey: ["stacks"] });
      queryClient.invalidateQueries({ queryKey: ["stack", stackId] });
    },
  });
}

export function useDeleteStack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (stackId: string) => api.deleteStack(stackId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["stacks"] });
    },
  });
}

export function useEnableStack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (stackId: string) => api.enableStack(stackId),
    onSuccess: (_data, stackId) => {
      queryClient.invalidateQueries({ queryKey: ["stacks"] });
      queryClient.invalidateQueries({ queryKey: ["stack", stackId] });
    },
  });
}

export function useDisableStack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (stackId: string) => api.disableStack(stackId),
    onSuccess: (_data, stackId) => {
      queryClient.invalidateQueries({ queryKey: ["stacks"] });
      queryClient.invalidateQueries({ queryKey: ["stack", stackId] });
    },
  });
}

export function useTriggerStack() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (cardId: string) => api.triggerCard(cardId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["executions"] });
    },
  });
}
