// Individual card item in the card list
import type { Card, TriggerConfig } from "../../types";
import { useCardStore } from "../../stores/cardStore";

interface CardItemProps {
  card: Card;
  onSelect?: (card: Card) => void;
}

export function CardItem({ card, onSelect }: CardItemProps) {
  const { enableCard, disableCard, triggerCard, deleteCard } = useCardStore();

  const getTriggerIcon = (trigger: TriggerConfig): string => {
    switch (trigger.type) {
      case "file_created":
        return "M12 4v16m8-8H4"; // Plus icon
      case "file_modified":
        return "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"; // Edit icon
      case "schedule":
        return "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"; // Clock icon
      case "manual":
        return "M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122"; // Cursor icon
    }
  };

  const getTriggerLabel = (trigger: TriggerConfig): string => {
    switch (trigger.type) {
      case "file_created":
        return `New files in ${trigger.watch_path}`;
      case "file_modified":
        return `Modified files in ${trigger.watch_path}`;
      case "schedule":
        return `Schedule: ${trigger.cron}`;
      case "manual":
        return "Manual trigger";
    }
  };

  const getPatterns = (trigger: TriggerConfig): string[] => {
    if (trigger.type === "file_created" || trigger.type === "file_modified") {
      return trigger.patterns;
    }
    return [];
  };

  const handleToggleEnabled = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      if (card.enabled) {
        await disableCard(card.id);
      } else {
        await enableCard(card.id);
      }
    } catch (error) {
      console.error("Failed to toggle card:", error);
    }
  };

  const handleTrigger = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await triggerCard(card.id);
    } catch (error) {
      console.error("Failed to trigger card:", error);
    }
  };

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(`Delete card "${card.name}"?`)) {
      try {
        await deleteCard(card.id);
      } catch (error) {
        console.error("Failed to delete card:", error);
      }
    }
  };

  const patterns = getPatterns(card.trigger);

  return (
    <div
      className={`px-4 py-3 hover:bg-gray-50 transition-colors cursor-pointer ${
        !card.enabled ? "opacity-60" : ""
      }`}
      onClick={() => onSelect?.(card)}
    >
      <div className="flex items-start gap-3">
        {/* Trigger type icon */}
        <div
          className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${
            card.enabled ? "bg-leaf-100" : "bg-gray-100"
          }`}
        >
          <svg
            className={`w-5 h-5 ${card.enabled ? "text-leaf-600" : "text-gray-400"}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d={getTriggerIcon(card.trigger)}
            />
          </svg>
        </div>

        {/* Card details */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-900 truncate">{card.name}</span>
            {!card.enabled && (
              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600">
                Disabled
              </span>
            )}
          </div>

          {card.description && (
            <p className="text-sm text-gray-500 truncate mt-0.5">{card.description}</p>
          )}

          <div className="flex items-center gap-3 mt-1 text-xs text-gray-400">
            <span>{getTriggerLabel(card.trigger)}</span>
            {patterns.length > 0 && (
              <span className="text-leaf-600">{patterns.join(", ")}</span>
            )}
          </div>
        </div>

        {/* Actions */}
        <div className="flex-shrink-0 flex items-center gap-1">
          {/* Toggle enabled */}
          <button
            onClick={handleToggleEnabled}
            className={`p-1.5 rounded transition-colors ${
              card.enabled
                ? "text-leaf-600 hover:bg-leaf-100"
                : "text-gray-400 hover:bg-gray-100"
            }`}
            title={card.enabled ? "Disable card" : "Enable card"}
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              {card.enabled ? (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              ) : (
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              )}
            </svg>
          </button>

          {/* Run now button (only for enabled cards) */}
          {card.enabled && (
            <button
              onClick={handleTrigger}
              className="p-1.5 rounded text-gray-400 hover:text-leaf-600 hover:bg-leaf-100 transition-colors"
              title="Run now"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </button>
          )}

          {/* Delete button */}
          <button
            onClick={handleDelete}
            className="p-1.5 rounded text-gray-400 hover:text-red-600 hover:bg-red-50 transition-colors"
            title="Delete card"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
