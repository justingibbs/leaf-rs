// Renders a stack proposal from the propose_stack tool result inline in chat
import { useState } from "react";
import { useChatStore } from "../../stores/chatStore";

interface CardPreview {
  position: number;
  name: string;
  description?: string | null;
  program_path: string;
  code_preview: string;
}

interface StackProposal {
  name: string;
  description?: string;
  trigger: {
    type: string;
    watch_path?: string;
    patterns?: string[];
  };
}

export interface StackProposalResult {
  status: "proposed";
  stack: StackProposal;
  cards: CardPreview[];
}

function formatTrigger(trigger: StackProposal["trigger"]): string {
  switch (trigger.type) {
    case "file_created": {
      const path = trigger.watch_path || ".";
      const patterns = trigger.patterns?.length
        ? trigger.patterns.join(", ")
        : "*";
      return `When files matching ${patterns} are created in ${path}`;
    }
    case "file_modified": {
      const path = trigger.watch_path || ".";
      const patterns = trigger.patterns?.length
        ? trigger.patterns.join(", ")
        : "*";
      return `When files matching ${patterns} are modified in ${path}`;
    }
    case "manual":
      return "Manual trigger (run from UI)";
    default:
      return trigger.type;
  }
}

interface StackProposalCardProps {
  proposal: StackProposalResult;
}

export function StackProposalCard({ proposal }: StackProposalCardProps) {
  const { sendMessage } = useChatStore();
  const [expandedCards, setExpandedCards] = useState<Set<number>>(new Set());
  const [acted, setActed] = useState<"approved" | "rejected" | null>(null);

  const toggleCard = (position: number) => {
    setExpandedCards((prev) => {
      const next = new Set(prev);
      if (next.has(position)) {
        next.delete(position);
      } else {
        next.add(position);
      }
      return next;
    });
  };

  const handleApprove = () => {
    setActed("approved");
    sendMessage("Yes, create this stack.");
  };

  const handleReject = () => {
    setActed("rejected");
    sendMessage("No, don't create this stack.");
  };

  const { stack, cards } = proposal;

  return (
    <div className="mt-2 rounded-lg border border-leaf-200 dark:border-leaf-800 bg-leaf-50/50 dark:bg-leaf-900/20 overflow-hidden">
      {/* Header */}
      <div className="px-3 py-2 bg-leaf-100 dark:bg-leaf-900/40 border-b border-leaf-200 dark:border-leaf-800">
        <div className="flex items-center gap-2">
          <svg className="w-4 h-4 text-leaf-600 dark:text-leaf-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <span className="text-sm font-medium text-leaf-700 dark:text-leaf-300">
            Stack Proposal
          </span>
        </div>
      </div>

      <div className="p-3 space-y-3">
        {/* Stack name & description */}
        <div>
          <h4 className="font-medium text-gray-900 dark:text-gray-100 text-sm">
            {stack.name}
          </h4>
          {stack.description && (
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              {stack.description}
            </p>
          )}
        </div>

        {/* Trigger */}
        <div className="flex items-center gap-1.5 text-xs text-gray-600 dark:text-gray-400">
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span>{formatTrigger(stack.trigger)}</span>
        </div>

        {/* Cards */}
        <div className="space-y-1.5">
          <div className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
            {cards.length} Card{cards.length !== 1 ? "s" : ""} in Pipeline
          </div>
          {cards.map((card) => (
            <div
              key={card.position}
              className="rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 overflow-hidden"
            >
              <button
                onClick={() => toggleCard(card.position)}
                className="w-full flex items-center gap-2 px-2.5 py-1.5 text-left hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors"
              >
                <span className="flex-shrink-0 w-5 h-5 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center text-xs font-medium text-gray-600 dark:text-gray-300">
                  {card.position + 1}
                </span>
                <span className="flex-1 text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
                  {card.name}
                </span>
                <svg
                  className={`w-3.5 h-3.5 text-gray-400 transition-transform ${expandedCards.has(card.position) ? "rotate-180" : ""}`}
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                </svg>
              </button>

              {expandedCards.has(card.position) && (
                <div className="border-t border-gray-200 dark:border-gray-700">
                  {card.description && (
                    <p className="px-2.5 py-1.5 text-xs text-gray-500 dark:text-gray-400 border-b border-gray-100 dark:border-gray-700">
                      {card.description}
                    </p>
                  )}
                  <pre className="px-2.5 py-2 text-xs font-mono text-gray-700 dark:text-gray-300 bg-gray-50 dark:bg-gray-900 overflow-x-auto max-h-48">
                    <code>{card.code_preview}</code>
                  </pre>
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Actions */}
        {acted === null ? (
          <div className="flex gap-2 pt-1">
            <button
              onClick={handleApprove}
              className="flex-1 px-3 py-1.5 text-sm font-medium text-white bg-leaf-600 rounded-md hover:bg-leaf-700 transition-colors"
            >
              Approve
            </button>
            <button
              onClick={handleReject}
              className="flex-1 px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
              Reject
            </button>
          </div>
        ) : (
          <div
            className={`text-xs text-center py-1.5 rounded-md ${
              acted === "approved"
                ? "bg-leaf-50 dark:bg-leaf-900/30 text-leaf-700 dark:text-leaf-400"
                : "bg-gray-50 dark:bg-gray-800 text-gray-500 dark:text-gray-400"
            }`}
          >
            {acted === "approved" ? "Approved" : "Rejected"}
          </div>
        )}
      </div>
    </div>
  );
}
