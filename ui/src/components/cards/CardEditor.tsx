// Card editor component for creating and editing cards
import { useState } from "react";
import type { Card, TriggerConfig, ProgramConfig, CreateCardInput, UpdateCardInput } from "../../types";
import { useCardStore } from "../../stores/cardStore";

interface CardEditorProps {
  card?: Card; // If provided, we're editing; otherwise, creating
  onSave?: (card: Card) => void;
  onCancel?: () => void;
}

type TriggerType = TriggerConfig["type"];

const defaultProgramConfig: ProgramConfig = {
  language: "typescript",
  entrypoint: "main.ts",
  dependencies: [],
  timeout_secs: 300,
  max_retries: 3,
};

export function CardEditor({ card, onSave, onCancel }: CardEditorProps) {
  const { createCard, updateCard } = useCardStore();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [name, setName] = useState(card?.name ?? "");
  const [description, setDescription] = useState(card?.description ?? "");
  const [triggerType, setTriggerType] = useState<TriggerType>(
    card?.trigger.type ?? "file_created"
  );
  const [watchPath, setWatchPath] = useState(
    (card?.trigger.type === "file_created" || card?.trigger.type === "file_modified"
      ? card.trigger.watch_path
      : "") ?? ""
  );
  const [patterns, setPatterns] = useState(
    (card?.trigger.type === "file_created" || card?.trigger.type === "file_modified"
      ? card.trigger.patterns.join(", ")
      : "") ?? ""
  );
  const [cronExpression, setCronExpression] = useState(
    card?.trigger.type === "schedule" ? card.trigger.cron : ""
  );
  const [timeoutSecs, setTimeoutSecs] = useState(
    card?.program.timeout_secs ?? defaultProgramConfig.timeout_secs
  );
  const [maxRetries, setMaxRetries] = useState(
    card?.program.max_retries ?? defaultProgramConfig.max_retries
  );

  const isEditing = !!card;

  const buildTriggerConfig = (): TriggerConfig => {
    switch (triggerType) {
      case "file_created":
        return {
          type: "file_created",
          watch_path: watchPath || ".",
          patterns: patterns
            .split(",")
            .map((p) => p.trim())
            .filter(Boolean),
        };
      case "file_modified":
        return {
          type: "file_modified",
          watch_path: watchPath || ".",
          patterns: patterns
            .split(",")
            .map((p) => p.trim())
            .filter(Boolean),
        };
      case "schedule":
        return {
          type: "schedule",
          cron: cronExpression || "0 0 * * *",
        };
      case "manual":
        return { type: "manual" };
    }
  };

  const buildProgramConfig = (): ProgramConfig => ({
    ...defaultProgramConfig,
    timeout_secs: timeoutSecs,
    max_retries: maxRetries,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError("Card name is required");
      return;
    }

    setIsSubmitting(true);

    try {
      if (isEditing) {
        const input: UpdateCardInput = {
          name: name.trim(),
          description: description.trim(),
          trigger: buildTriggerConfig(),
          program: buildProgramConfig(),
        };
        const updatedCard = await updateCard(card.id, input);
        onSave?.(updatedCard);
      } else {
        const input: CreateCardInput = {
          name: name.trim(),
          description: description.trim(),
          trigger: buildTriggerConfig(),
          program: buildProgramConfig(),
        };
        const newCard = await createCard(input);
        onSave?.(newCard);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="bg-white rounded-lg border border-gray-200">
      <div className="px-4 py-3 border-b border-gray-200">
        <h2 className="text-lg font-medium text-gray-900">
          {isEditing ? "Edit Card" : "New Card"}
        </h2>
      </div>

      <form onSubmit={handleSubmit} className="p-4 space-y-4">
        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-md text-sm text-red-700">
            {error}
          </div>
        )}

        {/* Basic info */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Name <span className="text-red-500">*</span>
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., CSV Analyzer"
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Description
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="What does this card do?"
            rows={2}
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500"
          />
        </div>

        {/* Trigger configuration */}
        <div className="pt-2 border-t border-gray-200">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Trigger
          </label>

          <div className="grid grid-cols-2 gap-2 mb-3">
            {(
              [
                { value: "file_created", label: "File Created" },
                { value: "file_modified", label: "File Modified" },
                { value: "schedule", label: "Schedule" },
                { value: "manual", label: "Manual" },
              ] as const
            ).map((option) => (
              <button
                key={option.value}
                type="button"
                onClick={() => setTriggerType(option.value)}
                className={`px-3 py-2 text-sm font-medium rounded-md border transition-colors ${
                  triggerType === option.value
                    ? "bg-leaf-50 border-leaf-500 text-leaf-700"
                    : "bg-white border-gray-300 text-gray-700 hover:bg-gray-50"
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>

          {/* File trigger options */}
          {(triggerType === "file_created" || triggerType === "file_modified") && (
            <div className="space-y-3 p-3 bg-gray-50 rounded-md">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Watch Path
                </label>
                <input
                  type="text"
                  value={watchPath}
                  onChange={(e) => setWatchPath(e.target.value)}
                  placeholder="e.g., inbox or /path/to/folder"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Relative to project root or absolute path
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  File Patterns
                </label>
                <input
                  type="text"
                  value={patterns}
                  onChange={(e) => setPatterns(e.target.value)}
                  placeholder="e.g., *.csv, *.pdf, report_*.xlsx"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Comma-separated glob patterns. Leave empty to match all files.
                </p>
              </div>
            </div>
          )}

          {/* Schedule trigger options */}
          {triggerType === "schedule" && (
            <div className="space-y-3 p-3 bg-gray-50 rounded-md">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Cron Expression
                </label>
                <input
                  type="text"
                  value={cronExpression}
                  onChange={(e) => setCronExpression(e.target.value)}
                  placeholder="0 0 * * *"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm font-mono"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Standard cron format: minute hour day month weekday
                </p>
              </div>
            </div>
          )}

          {/* Manual trigger info */}
          {triggerType === "manual" && (
            <div className="p-3 bg-gray-50 rounded-md">
              <p className="text-sm text-gray-600">
                This card will only run when manually triggered from the UI.
              </p>
            </div>
          )}
        </div>

        {/* Execution settings */}
        <div className="pt-2 border-t border-gray-200">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Execution Settings
          </label>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">
                Timeout (seconds)
              </label>
              <input
                type="number"
                value={timeoutSecs}
                onChange={(e) => setTimeoutSecs(Number(e.target.value))}
                min={10}
                max={3600}
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">
                Max Retries
              </label>
              <input
                type="number"
                value={maxRetries}
                onChange={(e) => setMaxRetries(Number(e.target.value))}
                min={0}
                max={10}
                className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm"
              />
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-2 pt-4 border-t border-gray-200">
          <button
            type="button"
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            className="px-4 py-2 text-sm font-medium text-white bg-leaf-600 border border-transparent rounded-md hover:bg-leaf-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? "Saving..." : isEditing ? "Save Changes" : "Create Card"}
          </button>
        </div>
      </form>
    </div>
  );
}
