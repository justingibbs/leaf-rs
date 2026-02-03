// Card store for managing automation cards
import { create } from "zustand";
import type { Card, CreateCardInput, UpdateCardInput } from "../types";
import { api } from "../lib/tauri";

interface CardState {
  // Cards list
  cards: Card[];
  isLoading: boolean;
  error: string | null;

  // Selected card for detail view
  selectedCardId: string | null;

  // Actions
  loadCards: () => Promise<void>;
  getCard: (cardId: string) => Promise<Card | null>;
  createCard: (input: CreateCardInput) => Promise<Card>;
  updateCard: (cardId: string, input: UpdateCardInput) => Promise<Card>;
  deleteCard: (cardId: string) => Promise<void>;
  enableCard: (cardId: string) => Promise<void>;
  disableCard: (cardId: string) => Promise<void>;
  triggerCard: (cardId: string) => Promise<void>;

  // Real-time update handlers (from LeafEvent)
  addCard: (card: Card) => void;
  updateCardInStore: (card: Card) => void;
  removeCard: (cardId: string) => void;

  // Selection
  selectCard: (cardId: string | null) => void;

  // Reset
  reset: () => void;
}

export const useCardStore = create<CardState>((set) => ({
  // Initial state
  cards: [],
  isLoading: false,
  error: null,
  selectedCardId: null,

  // Load all cards from the backend
  loadCards: async () => {
    set({ isLoading: true, error: null });
    try {
      const cards = await api.listCards();
      set({ cards, isLoading: false });
    } catch (error) {
      console.error("Failed to load cards:", error);
      set({ error: String(error), isLoading: false });
    }
  },

  // Get a single card by ID
  getCard: async (cardId: string) => {
    try {
      return await api.getCard(cardId);
    } catch (error) {
      console.error("Failed to get card:", error);
      set({ error: String(error) });
      return null;
    }
  },

  // Create a new card
  createCard: async (input: CreateCardInput) => {
    set({ isLoading: true, error: null });
    try {
      const card = await api.createCard(input);
      // The card will be added via real-time event, but also add optimistically
      set((state) => ({
        cards: [card, ...state.cards],
        isLoading: false,
      }));
      return card;
    } catch (error) {
      console.error("Failed to create card:", error);
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  // Update an existing card
  updateCard: async (cardId: string, input: UpdateCardInput) => {
    set({ error: null });
    try {
      const card = await api.updateCard(cardId, input);
      // The card will be updated via real-time event, but also update optimistically
      set((state) => ({
        cards: state.cards.map((c) => (c.id === cardId ? card : c)),
      }));
      return card;
    } catch (error) {
      console.error("Failed to update card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Delete a card
  deleteCard: async (cardId: string) => {
    set({ error: null });
    try {
      await api.deleteCard(cardId);
      // The card will be removed via real-time event, but also remove optimistically
      set((state) => ({
        cards: state.cards.filter((c) => c.id !== cardId),
        selectedCardId:
          state.selectedCardId === cardId ? null : state.selectedCardId,
      }));
    } catch (error) {
      console.error("Failed to delete card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Enable a card
  enableCard: async (cardId: string) => {
    set({ error: null });
    try {
      const card = await api.enableCard(cardId);
      set((state) => ({
        cards: state.cards.map((c) => (c.id === cardId ? card : c)),
      }));
    } catch (error) {
      console.error("Failed to enable card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Disable a card
  disableCard: async (cardId: string) => {
    set({ error: null });
    try {
      const card = await api.disableCard(cardId);
      set((state) => ({
        cards: state.cards.map((c) => (c.id === cardId ? card : c)),
      }));
    } catch (error) {
      console.error("Failed to disable card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Manually trigger a card
  triggerCard: async (cardId: string) => {
    set({ error: null });
    try {
      await api.triggerCard(cardId);
    } catch (error) {
      console.error("Failed to trigger card:", error);
      set({ error: String(error) });
      throw error;
    }
  },

  // Add a card (from real-time updates)
  addCard: (card: Card) => {
    set((state) => {
      // Don't add if already exists (optimistic update already added it)
      if (state.cards.some((c) => c.id === card.id)) {
        return state;
      }
      return {
        cards: [card, ...state.cards],
      };
    });
  },

  // Update a card in the store (from real-time updates)
  updateCardInStore: (card: Card) => {
    set((state) => ({
      cards: state.cards.map((c) => (c.id === card.id ? card : c)),
    }));
  },

  // Remove a card from the store (from real-time updates)
  removeCard: (cardId: string) => {
    set((state) => ({
      cards: state.cards.filter((c) => c.id !== cardId),
      selectedCardId:
        state.selectedCardId === cardId ? null : state.selectedCardId,
    }));
  },

  // Select a card for detail view
  selectCard: (cardId: string | null) => {
    set({ selectedCardId: cardId });
  },

  // Reset store state
  reset: () => {
    set({
      cards: [],
      isLoading: false,
      error: null,
      selectedCardId: null,
    });
  },
}));

// Helper to get a card by ID from the store
export const getCardById = (cardId: string): Card | undefined => {
  return useCardStore.getState().cards.find((c) => c.id === cardId);
};

// Helper to get enabled cards only
export const getEnabledCards = (): Card[] => {
  return useCardStore.getState().cards.filter((c) => c.enabled);
};
