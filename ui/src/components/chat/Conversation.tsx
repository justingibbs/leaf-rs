// Conversation component - displays messages and input
import { useRef, useEffect } from "react";
import { useChatStore } from "../../stores/chatStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { MessageBubble, ThinkingIndicator, ToolCallIndicator } from "./MessageBubble";
import { ChatInput } from "./ChatInput";

export function Conversation() {
  const {
    messages,
    messagesLoading,
    messagesError,
    isAgentThinking,
    currentToolCall,
    sendMessage,
    activeSessionId,
    agentConfig,
  } = useChatStore();

  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isAgentThinking, currentToolCall]);

  if (!activeSessionId) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500 dark:text-gray-400">
        <div className="text-center">
          <svg className="w-12 h-12 mx-auto mb-3 text-gray-300 dark:text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <p>Select a conversation or start a new one</p>
        </div>
      </div>
    );
  }

  // Check if API key is configured
  const hasApiKey = Boolean(agentConfig?.api_key);

  return (
    <div className="flex flex-col h-full">
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-4">
        {messagesError && (
          <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-800 rounded-lg text-sm text-red-700 dark:text-red-300">
            {messagesError}
          </div>
        )}

        {messagesLoading ? (
          <div className="text-center text-gray-500 dark:text-gray-400 py-8">
            Loading messages...
          </div>
        ) : messages.length === 0 ? (
          <div className="text-center py-8">
            <div className="inline-flex items-center justify-center w-16 h-16 bg-leaf-100 dark:bg-leaf-900/30 rounded-full mb-4">
              <svg className="w-8 h-8 text-leaf-600 dark:text-leaf-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
              Start Creating Automations
            </h3>
            <p className="text-gray-500 dark:text-gray-400 max-w-sm mx-auto">
              Describe what you want to automate and I'll help you create cards
              that process your files automatically.
            </p>
            <div className="mt-4 flex flex-wrap justify-center gap-2">
              <ExamplePrompt text="Create a card that summarizes PDF files" />
              <ExamplePrompt text="Process CSV files and extract data" />
              <ExamplePrompt text="Convert images to thumbnails" />
            </div>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
          </>
        )}

        {/* Tool call indicator */}
        {currentToolCall && (
          <ToolCallIndicator toolName={currentToolCall.name} />
        )}

        {/* Thinking indicator */}
        {isAgentThinking && !currentToolCall && <ThinkingIndicator />}

        {/* Scroll anchor */}
        <div ref={messagesEndRef} />
      </div>

      {/* API key warning */}
      {!hasApiKey && (
        <ApiKeyWarning />
      )}

      {/* Input area */}
      <ChatInput
        onSend={sendMessage}
        disabled={isAgentThinking || !hasApiKey}
        placeholder={
          !hasApiKey
            ? "Configure API key in Settings to chat..."
            : isAgentThinking
            ? "Waiting for response..."
            : undefined
        }
      />
    </div>
  );
}

// Example prompt button
function ExamplePrompt({ text }: { text: string }) {
  const { sendMessage, agentConfig } = useChatStore();

  const handleClick = () => {
    if (agentConfig?.api_key) {
      sendMessage(text);
    }
  };

  return (
    <button
      onClick={handleClick}
      className="px-3 py-1.5 text-xs text-gray-600 dark:text-gray-400 bg-gray-100 dark:bg-gray-800 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
    >
      {text}
    </button>
  );
}

// API key warning component
function ApiKeyWarning() {
  const { openSettings } = useSettingsStore();

  return (
    <div className="px-4 py-2 bg-amber-50 dark:bg-amber-900/30 border-t border-amber-200 dark:border-amber-800">
      <p className="text-sm text-amber-700 dark:text-amber-300">
        Configure your API key in{" "}
        <button
          onClick={() => openSettings("llm")}
          className="underline hover:no-underline"
        >
          Settings
        </button>{" "}
        to start chatting.
      </p>
    </div>
  );
}
