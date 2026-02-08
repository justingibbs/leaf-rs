// Visual progress tracker for multi-step stack creation (create_stack_now results)

export interface CreateStackResult {
  status: "created";
  stack_id: string;
  cards: Array<{
    card_id: string;
    name: string;
    position: number;
    program_path: string;
  }>;
}

interface WorkflowProgressProps {
  result: CreateStackResult;
}

function CheckIcon() {
  return (
    <svg className="w-4 h-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
    </svg>
  );
}

export function WorkflowProgress({ result }: WorkflowProgressProps) {
  const steps = [
    { label: "Stack created", done: true },
    ...result.cards.map((card) => ({
      label: `Card ${card.position + 1}: "${card.name}"`,
      done: true,
    })),
  ];

  return (
    <div className="mt-2 rounded-lg border border-green-200 dark:border-green-800 bg-green-50/50 dark:bg-green-900/20 overflow-hidden">
      {/* Header */}
      <div className="px-3 py-2 bg-green-100 dark:bg-green-900/40 border-b border-green-200 dark:border-green-800">
        <div className="flex items-center gap-2">
          <CheckIcon />
          <span className="text-sm font-medium text-green-700 dark:text-green-300">
            Workflow Created
          </span>
        </div>
      </div>

      {/* Steps */}
      <div className="p-3">
        <div className="space-y-1.5">
          {steps.map((step, i) => (
            <div key={i} className="flex items-center gap-2">
              <div className="flex-shrink-0">
                {step.done ? (
                  <CheckIcon />
                ) : (
                  <div className="w-4 h-4 rounded-full border-2 border-gray-300 dark:border-gray-600" />
                )}
              </div>
              <span
                className={`text-sm ${
                  step.done
                    ? "text-gray-700 dark:text-gray-300"
                    : "text-gray-400 dark:text-gray-500"
                }`}
              >
                {step.label}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
