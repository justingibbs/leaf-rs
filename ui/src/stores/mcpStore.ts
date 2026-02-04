import { create } from "zustand";
import type { McpServer, McpToolSummary, CreateMcpServerInput, McpServerStatus } from "../types";
import { api } from "../lib/tauri";

interface McpState {
  // Servers
  servers: McpServer[];
  serversLoading: boolean;
  serversError: string | null;

  // Tools
  tools: McpToolSummary[];
  toolsLoading: boolean;

  // Connection status (server_id -> status)
  connectionStatus: Map<string, McpServerStatus>;

  // Actions
  loadServers: () => Promise<void>;
  addServer: (input: CreateMcpServerInput) => Promise<McpServer>;
  removeServer: (serverId: string) => Promise<void>;
  enableServer: (serverId: string) => Promise<void>;
  disableServer: (serverId: string) => Promise<void>;
  testServer: (serverId: string) => Promise<McpServerStatus>;
  loadTools: () => Promise<void>;

  // Event handlers
  handleServerConnected: (serverId: string, toolCount: number) => void;
  handleServerDisconnected: (serverId: string) => void;
  handleServerError: (serverId: string, error: string) => void;
}

export const useMcpStore = create<McpState>((set, get) => ({
  // Initial state
  servers: [],
  serversLoading: false,
  serversError: null,

  tools: [],
  toolsLoading: false,

  connectionStatus: new Map(),

  // Load all servers
  loadServers: async () => {
    set({ serversLoading: true, serversError: null });
    try {
      const servers = await api.listMcpServers();
      set({ servers, serversLoading: false });
    } catch (error) {
      set({ serversError: String(error), serversLoading: false });
    }
  },

  // Add a new server
  addServer: async (input: CreateMcpServerInput) => {
    try {
      const server = await api.addMcpServer(input);
      set((state) => ({
        servers: [...state.servers, server],
      }));
      return server;
    } catch (error) {
      set({ serversError: String(error) });
      throw error;
    }
  },

  // Remove a server
  removeServer: async (serverId: string) => {
    try {
      await api.removeMcpServer(serverId);
      set((state) => ({
        servers: state.servers.filter((s) => s.id !== serverId),
      }));
    } catch (error) {
      set({ serversError: String(error) });
    }
  },

  // Enable a server
  enableServer: async (serverId: string) => {
    try {
      const updated = await api.enableMcpServer(serverId);
      set((state) => ({
        servers: state.servers.map((s) => (s.id === serverId ? updated : s)),
      }));
    } catch (error) {
      set({ serversError: String(error) });
    }
  },

  // Disable a server
  disableServer: async (serverId: string) => {
    try {
      const updated = await api.disableMcpServer(serverId);
      set((state) => ({
        servers: state.servers.map((s) => (s.id === serverId ? updated : s)),
      }));
    } catch (error) {
      set({ serversError: String(error) });
    }
  },

  // Test server connection
  testServer: async (serverId: string) => {
    try {
      const status = await api.testMcpServer(serverId);
      const newStatus = new Map(get().connectionStatus);
      newStatus.set(serverId, status);
      set({ connectionStatus: newStatus });
      return status;
    } catch (error) {
      const status: McpServerStatus = { connected: false, tool_count: 0 };
      const newStatus = new Map(get().connectionStatus);
      newStatus.set(serverId, status);
      set({ connectionStatus: newStatus });
      return status;
    }
  },

  // Load all available tools
  loadTools: async () => {
    set({ toolsLoading: true });
    try {
      const tools = await api.listMcpTools();
      set({ tools, toolsLoading: false });
    } catch (error) {
      set({ toolsLoading: false });
    }
  },

  // Event handlers
  handleServerConnected: (serverId: string, toolCount: number) => {
    const newStatus = new Map(get().connectionStatus);
    newStatus.set(serverId, { connected: true, tool_count: toolCount });
    set({ connectionStatus: newStatus });
  },

  handleServerDisconnected: (serverId: string) => {
    const newStatus = new Map(get().connectionStatus);
    newStatus.set(serverId, { connected: false, tool_count: 0 });
    set({ connectionStatus: newStatus });
  },

  handleServerError: (serverId: string, _error: string) => {
    const newStatus = new Map(get().connectionStatus);
    newStatus.set(serverId, { connected: false, tool_count: 0 });
    set({ connectionStatus: newStatus });
  },
}));
