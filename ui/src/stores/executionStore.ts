// Execution store for managing stack executions
import { create } from "zustand";
import type { StackExecution, ExecutionStatus } from "../types";
import { api } from "../lib/tauri";

interface ExecutionState {
  // Stack executions list
  executions: StackExecution[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadExecutions: (limit?: number) => Promise<void>;
  loadExecutionsForEvent: (eventId: string) => Promise<void>;
  getExecution: (executionId: string) => Promise<StackExecution | null>;

  // Real-time update handlers (from LeafEvent)
  addExecution: (execution: StackExecution) => void;
  updateExecution: (
    executionId: string,
    updates: Partial<StackExecution>
  ) => void;
  updateExecutionStatus: (
    executionId: string,
    status: ExecutionStatus
  ) => void;

  // Reset
  reset: () => void;
}

export const useExecutionStore = create<ExecutionState>((set) => ({
  executions: [],
  isLoading: false,
  error: null,

  loadExecutions: async (limit?: number) => {
    set({ isLoading: true, error: null });
    try {
      const executions = await api.listExecutions(limit);
      set({ executions, isLoading: false });
    } catch (error) {
      console.error("Failed to load executions:", error);
      set({ error: String(error), isLoading: false });
    }
  },

  loadExecutionsForEvent: async (eventId: string) => {
    set({ isLoading: true, error: null });
    try {
      const executions = await api.listExecutionsForEvent(eventId);
      set({ executions, isLoading: false });
    } catch (error) {
      console.error("Failed to load executions for event:", error);
      set({ error: String(error), isLoading: false });
    }
  },

  getExecution: async (executionId: string) => {
    try {
      return await api.getExecution(executionId);
    } catch (error) {
      console.error("Failed to get execution:", error);
      set({ error: String(error) });
      return null;
    }
  },

  addExecution: (execution: StackExecution) => {
    set((state) => {
      if (state.executions.some((e) => e.id === execution.id)) return state;
      return { executions: [execution, ...state.executions] };
    });
  },

  updateExecution: (
    executionId: string,
    updates: Partial<StackExecution>
  ) => {
    set((state) => ({
      executions: state.executions.map((e) =>
        e.id === executionId ? { ...e, ...updates } : e
      ),
    }));
  },

  updateExecutionStatus: (executionId: string, status: ExecutionStatus) => {
    set((state) => ({
      executions: state.executions.map((e) =>
        e.id === executionId
          ? {
              ...e,
              status,
              completed_at:
                status === "running"
                  ? e.completed_at
                  : new Date().toISOString(),
            }
          : e
      ),
    }));
  },

  reset: () => {
    set({
      executions: [],
      isLoading: false,
      error: null,
    });
  },
}));

// Helper to get executions for a specific stack
export const getExecutionsForStack = (stackId: string): StackExecution[] => {
  return useExecutionStore
    .getState()
    .executions.filter((e) => e.stack_id === stackId);
};

// Helper to get running executions
export const getRunningExecutions = (): StackExecution[] => {
  return useExecutionStore
    .getState()
    .executions.filter(
      (e) => e.status === "running" || e.status === "pending"
    );
};
