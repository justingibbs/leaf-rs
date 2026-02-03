// Card list component showing all automation cards
import { useEffect, useState } from "react";
import { useCardStore } from "../../stores/cardStore";
import { CardItem } from "./CardItem";
import type { Card } from "../../types";

interface CardListProps {
  onSelectCard?: (card: Card) => void;
  onCreateCard?: () => void;
}

export function CardList({ onSelectCard, onCreateCard }: CardListProps) {
  const { cards, isLoading, error, loadCards } = useCardStore();
  const [filter, setFilter] = useState<"all" | "enabled" | "disabled">("all");

  useEffect(() => {
    loadCards();
  }, [loadCards]);

  const filteredCards = cards.filter((card) => {
    if (filter === "enabled") return card.enabled;
    if (filter === "disabled") return !card.enabled;
    return true;
  });

  const enabledCount = cards.filter((c) => c.enabled).length;
  const disabledCount = cards.filter((c) => !c.enabled).length;

  return (
    <div className="bg-white rounded-lg border border-gray-200">
      <div className="px-4 py-3 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-medium text-gray-900">Cards</h2>
          <button
            onClick={onCreateCard}
            className="inline-flex items-center px-3 py-1.5 text-sm font-medium rounded-md text-white bg-leaf-600 hover:bg-leaf-700 transition-colors"
          >
            <svg
              className="w-4 h-4 mr-1.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 4v16m8-8H4"
              />
            </svg>
            New Card
          </button>
        </div>

        {/* Filter tabs */}
        {cards.length > 0 && (
          <div className="flex gap-2 mt-3">
            <button
              onClick={() => setFilter("all")}
              className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                filter === "all"
                  ? "bg-gray-900 text-white"
                  : "bg-gray-100 text-gray-600 hover:bg-gray-200"
              }`}
            >
              All ({cards.length})
            </button>
            <button
              onClick={() => setFilter("enabled")}
              className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                filter === "enabled"
                  ? "bg-leaf-600 text-white"
                  : "bg-gray-100 text-gray-600 hover:bg-gray-200"
              }`}
            >
              Enabled ({enabledCount})
            </button>
            <button
              onClick={() => setFilter("disabled")}
              className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                filter === "disabled"
                  ? "bg-gray-600 text-white"
                  : "bg-gray-100 text-gray-600 hover:bg-gray-200"
              }`}
            >
              Disabled ({disabledCount})
            </button>
          </div>
        )}
      </div>

      <div className="divide-y divide-gray-100 max-h-[500px] overflow-y-auto">
        {isLoading ? (
          <div className="p-4 text-center text-gray-500">
            <div className="animate-pulse">Loading cards...</div>
          </div>
        ) : error ? (
          <div className="p-4 text-center text-red-500">
            <p className="text-sm">Failed to load cards</p>
            <p className="text-xs mt-1">{error}</p>
          </div>
        ) : filteredCards.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
            <p className="mt-2 text-sm">
              {filter === "all"
                ? "No cards yet"
                : `No ${filter} cards`}
            </p>
            {filter === "all" && (
              <p className="text-xs text-gray-400 mt-1">
                Create a card to automate tasks when files are added
              </p>
            )}
          </div>
        ) : (
          filteredCards.map((card) => (
            <CardItem key={card.id} card={card} onSelect={onSelectCard} />
          ))
        )}
      </div>

      {filteredCards.length > 0 && (
        <div className="px-4 py-2 border-t border-gray-200 bg-gray-50">
          <p className="text-xs text-gray-500 text-center">
            Showing {filteredCards.length} of {cards.length} card
            {cards.length !== 1 ? "s" : ""}
          </p>
        </div>
      )}
    </div>
  );
}
