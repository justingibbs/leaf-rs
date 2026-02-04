// Execution store for managing card executions
import { create } from "zustand";
import type { Execution, ExecutionStatus } from "../types";
import { api } from "../lib/tauri";

interface ExecutionState {
  // Executions list
  executions: Execution[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadExecutions: (cardId?: string, limit?: number) => Promise<void>;
  loadExecutionsForEvent: (eventId: string) => Promise<void>;
  getExecution: (executionId: string) => Promise<Execution | null>;

  // Real-time update handlers (from LeafEvent)
  addExecution: (execution: Execution) => void;
  updateExecution: (executionId: string, updates: Partial<Execution>) => void;
  updateExecutionStatus: (
    executionId: string,
    status: ExecutionStatus,
    exitCode?: number | null
  ) => void;

  // Reset
  reset: () => void;
}

export const useExecutionStore = create<ExecutionState>((set) => ({
  // Initial state
  executions: [],
  isLoading: false,
  error: null,

  // Load executions from the backend
  loadExecutions: async (cardId?: string, limit?: number) => {
    set({ isLoading: true, error: null });
    try {
      const executions = await api.listExecutions(cardId, limit);
      set({ executions, isLoading: false });
    } catch (error) {
      console.error("Failed to load executions:", error);
      set({ error: String(error), isLoading: false });
    }
  },

  // Load executions for a specific event
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

  // Get a single execution by ID
  getExecution: async (executionId: string) => {
    try {
      return await api.getExecution(executionId);
    } catch (error) {
      console.error("Failed to get execution:", error);
      set({ error: String(error) });
      return null;
    }
  },

  // Add an execution (from real-time updates)
  addExecution: (execution: Execution) => {
    set((state) => {
      // Don't add if already exists
      if (state.executions.some((e) => e.id === execution.id)) {
        return state;
      }
      return {
        executions: [execution, ...state.executions],
      };
    });
  },

  // Update an execution (from real-time updates)
  updateExecution: (executionId: string, updates: Partial<Execution>) => {
    set((state) => ({
      executions: state.executions.map((e) =>
        e.id === executionId ? { ...e, ...updates } : e
      ),
    }));
  },

  // Update execution status (from completion events)
  updateExecutionStatus: (
    executionId: string,
    status: ExecutionStatus,
    exitCode?: number | null
  ) => {
    set((state) => ({
      executions: state.executions.map((e) =>
        e.id === executionId
          ? {
              ...e,
              status,
              exit_code: exitCode !== undefined ? exitCode : e.exit_code,
              completed_at:
                status === "running" ? e.completed_at : new Date().toISOString(),
            }
          : e
      ),
    }));
  },

  // Reset store state
  reset: () => {
    set({
      executions: [],
      isLoading: false,
      error: null,
    });
  },
}));

// Helper to get executions for a specific card
export const getExecutionsForCard = (cardId: string): Execution[] => {
  return useExecutionStore
    .getState()
    .executions.filter((e) => e.card_id === cardId);
};

// Helper to get running executions
export const getRunningExecutions = (): Execution[] => {
  return useExecutionStore
    .getState()
    .executions.filter((e) => e.status === "running" || e.status === "pending");
};

// Helper to check if a card has any running executions
export const isCardRunning = (cardId: string): boolean => {
  return useExecutionStore
    .getState()
    .executions.some(
      (e) =>
        e.card_id === cardId &&
        (e.status === "running" || e.status === "pending")
    );
};
