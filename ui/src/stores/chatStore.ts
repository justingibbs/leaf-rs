import { create } from "zustand";
import type { ChatSession, Message } from "../types";
import { api, type AgentConfig } from "../lib/tauri";

interface ChatState {
  // Sessions
  sessions: ChatSession[];
  activeSessionId: string | null;
  sessionsLoading: boolean;
  sessionsError: string | null;

  // Messages
  messages: Message[];
  messagesLoading: boolean;
  messagesError: string | null;

  // Agent state
  isAgentThinking: boolean;
  currentToolCall: { name: string; arguments: unknown } | null;

  // Agent configuration (loaded from project settings)
  agentConfig: AgentConfig | null;

  // Session actions
  loadSessions: () => Promise<void>;
  createSession: (title?: string) => Promise<ChatSession>;
  selectSession: (sessionId: string) => Promise<void>;
  archiveSession: (sessionId: string) => Promise<void>;
  deleteSession: (sessionId: string) => Promise<void>;
  updateSessionTitle: (sessionId: string, title: string) => Promise<void>;

  // Message actions
  loadMessages: (sessionId: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;

  // Real-time event handlers
  handleSessionCreated: (session: ChatSession) => void;
  handleSessionUpdated: (session: ChatSession) => void;
  handleMessageReceived: (message: Message) => void;
  handleAgentThinking: (sessionId: string) => void;
  handleAgentToolCall: (sessionId: string, toolName: string, args: unknown) => void;
  handleAgentDone: () => void;

  // Config actions
  setAgentConfig: (config: AgentConfig) => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  // Initial state
  sessions: [],
  activeSessionId: null,
  sessionsLoading: false,
  sessionsError: null,

  messages: [],
  messagesLoading: false,
  messagesError: null,

  isAgentThinking: false,
  currentToolCall: null,

  agentConfig: null,

  // Load all sessions
  loadSessions: async () => {
    set({ sessionsLoading: true, sessionsError: null });
    try {
      const sessions = await api.listActiveSessions();
      set({ sessions, sessionsLoading: false });

      // If we have sessions but none selected, select the most recent
      if (sessions.length > 0 && !get().activeSessionId) {
        const mostRecent = sessions[0];
        await get().selectSession(mostRecent.id);
      }
    } catch (error) {
      set({ sessionsError: String(error), sessionsLoading: false });
    }
  },

  // Create a new session
  createSession: async (title?: string) => {
    try {
      const session = await api.createSession(title);
      set((state) => ({
        sessions: [session, ...state.sessions],
        activeSessionId: session.id,
        messages: [],
      }));
      return session;
    } catch (error) {
      set({ sessionsError: String(error) });
      throw error;
    }
  },

  // Select a session and load its messages
  selectSession: async (sessionId: string) => {
    set({ activeSessionId: sessionId, isAgentThinking: false, currentToolCall: null });
    await get().loadMessages(sessionId);
  },

  // Archive a session
  archiveSession: async (sessionId: string) => {
    try {
      await api.archiveSession(sessionId);
      set((state) => ({
        sessions: state.sessions.filter((s) => s.id !== sessionId),
        activeSessionId: state.activeSessionId === sessionId ? null : state.activeSessionId,
        messages: state.activeSessionId === sessionId ? [] : state.messages,
      }));
    } catch (error) {
      set({ sessionsError: String(error) });
    }
  },

  // Delete a session
  deleteSession: async (sessionId: string) => {
    try {
      await api.deleteSession(sessionId);
      set((state) => ({
        sessions: state.sessions.filter((s) => s.id !== sessionId),
        activeSessionId: state.activeSessionId === sessionId ? null : state.activeSessionId,
        messages: state.activeSessionId === sessionId ? [] : state.messages,
      }));
    } catch (error) {
      set({ sessionsError: String(error) });
    }
  },

  // Update session title
  updateSessionTitle: async (sessionId: string, title: string) => {
    try {
      const updated = await api.updateSessionTitle(sessionId, title);
      set((state) => ({
        sessions: state.sessions.map((s) => (s.id === sessionId ? updated : s)),
      }));
    } catch (error) {
      set({ sessionsError: String(error) });
    }
  },

  // Load messages for a session
  loadMessages: async (sessionId: string) => {
    set({ messagesLoading: true, messagesError: null });
    try {
      const messages = await api.getMessages(sessionId);
      set({ messages, messagesLoading: false });
    } catch (error) {
      set({ messagesError: String(error), messagesLoading: false });
    }
  },

  // Send a message to the agent
  sendMessage: async (content: string) => {
    const { activeSessionId, agentConfig } = get();

    if (!activeSessionId) {
      set({ messagesError: "No session selected" });
      return;
    }

    if (!agentConfig?.api_key) {
      set({ messagesError: "Please configure your API key in Settings" });
      return;
    }

    set({ isAgentThinking: true, currentToolCall: null, messagesError: null });

    try {
      // Send returns the user message immediately
      // Agent response comes through events
      const userMessage = await api.sendMessage(activeSessionId, content, agentConfig);

      // Add user message to the list
      set((state) => ({
        messages: [...state.messages, userMessage],
      }));
    } catch (error) {
      set({ messagesError: String(error), isAgentThinking: false });
    }
  },

  // Event handlers
  handleSessionCreated: (session: ChatSession) => {
    set((state) => ({
      sessions: [session, ...state.sessions.filter((s) => s.id !== session.id)],
    }));
  },

  handleSessionUpdated: (session: ChatSession) => {
    set((state) => ({
      sessions: state.sessions.map((s) => (s.id === session.id ? session : s)),
    }));
  },

  handleMessageReceived: (message: Message) => {
    const { activeSessionId } = get();

    // Only add if it's for the active session
    if (message.session_id === activeSessionId) {
      set((state) => {
        // Check if message already exists (might be updating)
        const exists = state.messages.some((m) => m.id === message.id);
        if (exists) {
          return {
            messages: state.messages.map((m) => (m.id === message.id ? message : m)),
            isAgentThinking: false,
          };
        }
        return {
          messages: [...state.messages, message],
          isAgentThinking: false,
        };
      });
    }
  },

  handleAgentThinking: (sessionId: string) => {
    const { activeSessionId } = get();
    if (sessionId === activeSessionId) {
      set({ isAgentThinking: true });
    }
  },

  handleAgentToolCall: (sessionId: string, toolName: string, args: unknown) => {
    const { activeSessionId } = get();
    if (sessionId === activeSessionId) {
      set({ currentToolCall: { name: toolName, arguments: args } });
    }
  },

  handleAgentDone: () => {
    set({ isAgentThinking: false, currentToolCall: null });
  },

  // Set agent config
  setAgentConfig: (config: AgentConfig) => {
    set({ agentConfig: config });
  },
}));
