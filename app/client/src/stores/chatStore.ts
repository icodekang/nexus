import { create } from 'zustand';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

export interface Conversation {
  id: string;
  title: string;
  model: string;
  messages: Message[];
  createdAt: number;
}

interface ChatState {
  conversations: Conversation[];
  activeConversationId: string | null;
  isLoading: boolean;
  selectedModel: string;
  showHistory: boolean;

  setSelectedModel: (model: string) => void;
  createConversation: (model: string, title?: string) => string;
  deleteConversation: (id: string) => void;
  setActiveConversation: (id: string) => void;
  addMessage: (conversationId: string, message: Omit<Message, 'id' | 'timestamp'>) => void;
  updateLastAssistantMessage: (conversationId: string, content: string) => void;
  setLoading: (loading: boolean) => void;
  setShowHistory: (show: boolean) => void;
  getActiveConversation: () => Conversation | undefined;
}

let messageCounter = 0;

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  isLoading: false,
  selectedModel: 'gpt-4o',
  showHistory: false,

  setSelectedModel: (model) => set({ selectedModel: model }),

  createConversation: (model, title) => {
    const id = `conv_${Date.now()}`;
    const conversation: Conversation = {
      id,
      title: title || 'New Chat',
      model,
      messages: [],
      createdAt: Date.now(),
    };
    set((state) => ({
      conversations: [conversation, ...state.conversations],
      activeConversationId: id,
    }));
    return id;
  },

  deleteConversation: (id) => set((state) => {
    const filtered = state.conversations.filter((c) => c.id !== id);
    return {
      conversations: filtered,
      activeConversationId:
        state.activeConversationId === id
          ? filtered[0]?.id || null
          : state.activeConversationId,
    };
  }),

  setActiveConversation: (id) => set({ activeConversationId: id }),

  addMessage: (conversationId, message) => set((state) => ({
    conversations: state.conversations.map((c) => {
      if (c.id !== conversationId) return c;
      const newMessage: Message = {
        ...message,
        id: `msg_${++messageCounter}_${Date.now()}`,
        timestamp: Date.now(),
      };
      const messages = [...c.messages, newMessage];
      // Auto-title from first user message
      const title = message.role === 'user' && c.messages.length === 0
        ? message.content.slice(0, 40) + (message.content.length > 40 ? '...' : '')
        : c.title;
      return { ...c, messages, title };
    }),
  })),

  updateLastAssistantMessage: (conversationId, content) => set((state) => ({
    conversations: state.conversations.map((c) => {
      if (c.id !== conversationId) return c;
      const messages = [...c.messages];
      for (let i = messages.length - 1; i >= 0; i--) {
        if (messages[i].role === 'assistant') {
          messages[i] = { ...messages[i], content };
          break;
        }
      }
      return { ...c, messages };
    }),
  })),

  setLoading: (loading) => set({ isLoading: loading }),

  setShowHistory: (show) => set({ showHistory: show }),

  getActiveConversation: () => {
    const state = get();
    return state.conversations.find((c) => c.id === state.activeConversationId);
  },
}));
