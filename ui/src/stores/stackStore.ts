// Stack store for managing automation stacks
import { create } from "zustand";
import type {
  Stack,
  Card,
  CreateStackInput,
  UpdateStackInput,
  CreateCardInput,
  UpdateCardInput,
} from "../types";
import { api } from "../lib/tauri";

interface StackState {
  // Stacks list
  stacks: Stack[];
  isLoading: boolean;
  error: string | null;

  // Cards per stack (keyed by stack_id)
  cardsByStack: Record<string, Card[]>;

  // Selected stack for detail view
  selectedStackId: string | null;

  // Stack actions
  loadStacks: () => Promise<void>;
  getStack: (stackId: string) => Promise<Stack | null>;
  createStack: (input: CreateStackInput) => Promise<Stack>;
  updateStack: (stackId: string, input: UpdateStackInput) => Promise<Stack>;
  deleteStack: (stackId: string) => Promise<void>;
  enableStack: (stackId: string) => Promise<void>;
  disableStack: (stackId: string) => Promise<void>;
  triggerStack: (stackId: string) => Promise<void>;

  // Card actions
  loadCards: (stackId: string) => Promise<void>;
  createCard: (input: CreateCardInput) => Promise<Card>;
  updateCard: (cardId: string, input: UpdateCardInput) => Promise<Card>;
  deleteCard: (cardId: string) => Promise<void>;

  // Real-time update handlers (from LeafEvent)
  addStack: (stack: Stack) => void;
  updateStackInStore: (stack: Stack) => void;
  removeStack: (stackId: string) => void;
  setStackEnabled: (stackId: string, enabled: boolean) => void;
  addCard: (card: Card) => void;
  updateCardInStore: (card: Card) => void;
  removeCard: (cardId: string, stackId: string) => void;
  reloadCardsForStack: (stackId: string) => void;

  // Selection
  selectStack: (stackId: string | null) => void;

  // Reset
  reset: () => void;
}

export const useStackStore = create<StackState>((set, get) => ({
  stacks: [],
  isLoading: false,
  error: null,
  cardsByStack: {},
  selectedStackId: null,

  loadStacks: async () => {
    set({ isLoading: true, error: null });
    try {
      const stacks = await api.listStacks();
      set({ stacks, isLoading: false });
    } catch (error) {
      console.error("Failed to load stacks:", error);
      set({ error: String(error), isLoading: false });
    }
  },

  getStack: async (stackId: string) => {
    try {
      return await api.getStack(stackId);
    } catch (error) {
      console.error("Failed to get stack:", error);
      set({ error: String(error) });
      return null;
    }
  },

  createStack: async (input: CreateStackInput) => {
    set({ isLoading: true, error: null });
    try {
      const stack = await api.createStack(input);
      set((state) => ({
        stacks: [stack, ...state.stacks],
        isLoading: false,
      }));
      return stack;
    } catch (error) {
      console.error("Failed to create stack:", error);
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  updateStack: async (stackId: string, input: UpdateStackInput) => {
    set({ error: null });
    try {
      const stack = await api.updateStack(stackId, input);
      set((state) => ({
        stacks: state.stacks.map((s) => (s.id === stackId ? stack : s)),
      }));
      return stack;
    } catch (error) {
      console.error("Failed to update stack:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  deleteStack: async (stackId: string) => {
    set({ error: null });
    try {
      await api.deleteStack(stackId);
      set((state) => {
        const { [stackId]: _, ...remainingCards } = state.cardsByStack;
        return {
          stacks: state.stacks.filter((s) => s.id !== stackId),
          cardsByStack: remainingCards,
          selectedStackId:
            state.selectedStackId === stackId ? null : state.selectedStackId,
        };
      });
    } catch (error) {
      console.error("Failed to delete stack:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  enableStack: async (stackId: string) => {
    set({ error: null });
    try {
      const stack = await api.enableStack(stackId);
      set((state) => ({
        stacks: state.stacks.map((s) => (s.id === stackId ? stack : s)),
      }));
    } catch (error) {
      console.error("Failed to enable stack:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  disableStack: async (stackId: string) => {
    set({ error: null });
    try {
      const stack = await api.disableStack(stackId);
      set((state) => ({
        stacks: state.stacks.map((s) => (s.id === stackId ? stack : s)),
      }));
    } catch (error) {
      console.error("Failed to disable stack:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  triggerStack: async (stackId: string) => {
    set({ error: null });
    try {
      // Get the first card in the stack to trigger
      const cards = get().cardsByStack[stackId];
      if (!cards || cards.length === 0) {
        throw new Error("Stack has no cards to trigger");
      }
      await api.triggerCard(cards[0].id);
    } catch (error) {
      console.error("Failed to trigger stack:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Card actions
  loadCards: async (stackId: string) => {
    try {
      const cards = await api.listCards(stackId);
      set((state) => ({
        cardsByStack: { ...state.cardsByStack, [stackId]: cards },
      }));
    } catch (error) {
      console.error("Failed to load cards for stack:", error);
      set({ error: String(error) });
    }
  },

  createCard: async (input: CreateCardInput) => {
    set({ error: null });
    try {
      const card = await api.createCard(input);
      set((state) => {
        const existing = state.cardsByStack[input.stack_id] ?? [];
        return {
          cardsByStack: {
            ...state.cardsByStack,
            [input.stack_id]: [...existing, card],
          },
        };
      });
      return card;
    } catch (error) {
      console.error("Failed to create card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  updateCard: async (cardId: string, input: UpdateCardInput) => {
    set({ error: null });
    try {
      const card = await api.updateCard(cardId, input);
      set((state) => {
        const stackCards = state.cardsByStack[card.stack_id] ?? [];
        return {
          cardsByStack: {
            ...state.cardsByStack,
            [card.stack_id]: stackCards.map((c) =>
              c.id === cardId ? card : c
            ),
          },
        };
      });
      return card;
    } catch (error) {
      console.error("Failed to update card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  deleteCard: async (cardId: string) => {
    set({ error: null });
    try {
      await api.deleteCard(cardId);
      // Remove from all stack card lists
      set((state) => {
        const updated: Record<string, Card[]> = {};
        for (const [stackId, cards] of Object.entries(state.cardsByStack)) {
          updated[stackId] = cards.filter((c) => c.id !== cardId);
        }
        return { cardsByStack: updated };
      });
    } catch (error) {
      console.error("Failed to delete card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Real-time event handlers
  addStack: (stack: Stack) => {
    set((state) => {
      if (state.stacks.some((s) => s.id === stack.id)) return state;
      return { stacks: [stack, ...state.stacks] };
    });
  },

  updateStackInStore: (stack: Stack) => {
    set((state) => ({
      stacks: state.stacks.map((s) => (s.id === stack.id ? stack : s)),
    }));
  },

  removeStack: (stackId: string) => {
    set((state) => {
      const { [stackId]: _, ...remainingCards } = state.cardsByStack;
      return {
        stacks: state.stacks.filter((s) => s.id !== stackId),
        cardsByStack: remainingCards,
        selectedStackId:
          state.selectedStackId === stackId ? null : state.selectedStackId,
      };
    });
  },

  setStackEnabled: (stackId: string, enabled: boolean) => {
    set((state) => ({
      stacks: state.stacks.map((s) =>
        s.id === stackId ? { ...s, enabled } : s
      ),
    }));
  },

  addCard: (card: Card) => {
    set((state) => {
      const existing = state.cardsByStack[card.stack_id] ?? [];
      if (existing.some((c) => c.id === card.id)) return state;
      return {
        cardsByStack: {
          ...state.cardsByStack,
          [card.stack_id]: [...existing, card],
        },
      };
    });
  },

  updateCardInStore: (card: Card) => {
    set((state) => {
      const stackCards = state.cardsByStack[card.stack_id] ?? [];
      return {
        cardsByStack: {
          ...state.cardsByStack,
          [card.stack_id]: stackCards.map((c) =>
            c.id === card.id ? card : c
          ),
        },
      };
    });
  },

  removeCard: (cardId: string, stackId: string) => {
    set((state) => {
      const stackCards = state.cardsByStack[stackId] ?? [];
      return {
        cardsByStack: {
          ...state.cardsByStack,
          [stackId]: stackCards.filter((c) => c.id !== cardId),
        },
      };
    });
  },

  reloadCardsForStack: (stackId: string) => {
    get().loadCards(stackId);
  },

  selectStack: (stackId: string | null) => {
    set({ selectedStackId: stackId });
  },

  reset: () => {
    set({
      stacks: [],
      isLoading: false,
      error: null,
      cardsByStack: {},
      selectedStackId: null,
    });
  },
}));
