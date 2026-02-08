// Stack detail view with card pipeline, trigger config, and recent runs
import { useState, useEffect } from "react";
import type {
  Stack,
  Card,
  TriggerConfig,
  CreateStackInput,
  UpdateStackInput,
} from "../../types";
import { useStackStore } from "../../stores/stackStore";
import { useExecutionStore } from "../../stores/executionStore";
import { CardStep } from "./CardStep";
import { StackExecutionView } from "./StackExecutionView";

interface StackDetailProps {
  stack?: Stack;
  onSave?: (stack: Stack) => void;
  onCancel?: () => void;
}

type TriggerType = TriggerConfig["type"];

export function StackDetail({ stack, onSave, onCancel }: StackDetailProps) {
  const {
    createStack,
    updateStack,
    cardsByStack,
    loadCards,
    updateCard,
    deleteCard,
  } = useStackStore();
  const { executions, loadExecutions } = useExecutionStore();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [name, setName] = useState(stack?.name ?? "");
  const [description, setDescription] = useState(stack?.description ?? "");
  const [triggerType, setTriggerType] = useState<TriggerType>(
    stack?.trigger.type ?? "file_created"
  );
  const [watchPath, setWatchPath] = useState(
    stack?.trigger.type === "file_created" ||
      stack?.trigger.type === "file_modified"
      ? stack.trigger.watch_path
      : ""
  );
  const [patterns, setPatterns] = useState(
    stack?.trigger.type === "file_created" ||
      stack?.trigger.type === "file_modified"
      ? stack.trigger.patterns.join(", ")
      : ""
  );
  const [cronExpression, setCronExpression] = useState(
    stack?.trigger.type === "schedule" ? stack.trigger.cron : ""
  );

  const isEditing = !!stack;
  const cards: Card[] = stack ? (cardsByStack[stack.id] ?? []) : [];
  const stackExecutions = stack
    ? executions.filter((e) => e.stack_id === stack.id)
    : [];

  useEffect(() => {
    if (stack) {
      loadCards(stack.id);
      loadExecutions(20);
    }
  }, [stack, loadCards, loadExecutions]);

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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError("Stack name is required");
      return;
    }

    setIsSubmitting(true);

    try {
      if (isEditing) {
        const input: UpdateStackInput = {
          name: name.trim(),
          description: description.trim(),
          trigger: buildTriggerConfig(),
        };
        const updatedStack = await updateStack(stack.id, input);
        onSave?.(updatedStack);
      } else {
        const input: CreateStackInput = {
          name: name.trim(),
          description: description.trim(),
          trigger: buildTriggerConfig(),
        };
        const newStack = await createStack(input);
        onSave?.(newStack);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleMoveCard = async (card: Card, direction: "up" | "down") => {
    const newPosition =
      direction === "up" ? card.position - 1 : card.position + 1;
    // Find the card at the target position and swap
    const targetCard = cards.find((c) => c.position === newPosition);
    if (targetCard) {
      await updateCard(targetCard.id, { position: card.position });
      await updateCard(card.id, { position: newPosition });
      if (stack) loadCards(stack.id);
    }
  };

  const handleDeleteCard = async (cardId: string) => {
    if (confirm("Remove this card from the stack?")) {
      await deleteCard(cardId);
    }
  };

  const sortedCards = [...cards].sort((a, b) => a.position - b.position);

  return (
    <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-lg font-medium text-gray-900 dark:text-gray-100">
          {isEditing ? "Edit Stack" : "New Stack"}
        </h2>
      </div>

      <div className="p-4 space-y-6 max-h-[calc(100vh-200px)] overflow-y-auto">
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-sm text-red-700 dark:text-red-400">
              {error}
            </div>
          )}

          {/* Basic info */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Name <span className="text-red-500">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., CSV Processing Pipeline"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What does this stack do?"
              rows={2}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
            />
          </div>

          {/* Trigger configuration */}
          <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
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
                      ? "bg-leaf-50 dark:bg-leaf-900/30 border-leaf-500 text-leaf-700 dark:text-leaf-400"
                      : "bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>

            {(triggerType === "file_created" ||
              triggerType === "file_modified") && (
              <div className="space-y-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-md">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Watch Path
                  </label>
                  <input
                    type="text"
                    value={watchPath}
                    onChange={(e) => setWatchPath(e.target.value)}
                    placeholder="e.g., inbox or /path/to/folder"
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Relative to project root or absolute path
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    File Patterns
                  </label>
                  <input
                    type="text"
                    value={patterns}
                    onChange={(e) => setPatterns(e.target.value)}
                    placeholder="e.g., *.csv, *.pdf, report_*.xlsx"
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Comma-separated glob patterns. Leave empty to match all
                    files.
                  </p>
                </div>
              </div>
            )}

            {triggerType === "schedule" && (
              <div className="space-y-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-md">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Cron Expression
                  </label>
                  <input
                    type="text"
                    value={cronExpression}
                    onChange={(e) => setCronExpression(e.target.value)}
                    placeholder="0 0 * * *"
                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-leaf-500 focus:border-leaf-500 text-sm font-mono bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                  />
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    Standard cron format: minute hour day month weekday
                  </p>
                </div>
              </div>
            )}

            {triggerType === "manual" && (
              <div className="p-3 bg-gray-50 dark:bg-gray-800 rounded-md">
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  This stack will only run when manually triggered from the UI.
                </p>
              </div>
            )}
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-4 py-2 text-sm font-medium text-white bg-leaf-600 border border-transparent rounded-md hover:bg-leaf-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting
                ? "Saving..."
                : isEditing
                  ? "Save Changes"
                  : "Create Stack"}
            </button>
          </div>
        </form>

        {/* Card Pipeline (only for existing stacks) */}
        {isEditing && (
          <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              Card Pipeline ({cards.length} card
              {cards.length !== 1 ? "s" : ""})
            </h3>

            {sortedCards.length === 0 ? (
              <div className="p-4 text-center text-gray-500 dark:text-gray-400 text-sm bg-gray-50 dark:bg-gray-800 rounded-lg">
                No cards in this stack. Use the chat to generate cards.
              </div>
            ) : (
              <div className="space-y-2">
                {sortedCards.map((card, idx) => (
                  <CardStep
                    key={card.id}
                    card={card}
                    position={idx}
                    totalCards={sortedCards.length}
                    onMoveUp={() => handleMoveCard(card, "up")}
                    onMoveDown={() => handleMoveCard(card, "down")}
                    onDelete={() => handleDeleteCard(card.id)}
                  />
                ))}
              </div>
            )}
          </div>
        )}

        {/* Recent Executions (only for existing stacks) */}
        {isEditing && stackExecutions.length > 0 && (
          <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
              Recent Runs
            </h3>
            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
              <StackExecutionView executions={stackExecutions.slice(0, 5)} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
