import { create } from 'zustand';

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

export interface Conversation {
  id: string;
  title: string;
  model: string;
  messages: Message[];
  createdAt: number;
  updatedAt: number;
}

interface ChatState {
  conversations: Conversation[];
  activeConversationId: string | null;
  isLoading: boolean;
  isStreaming: boolean;
  
  // Actions
  createConversation: (model: string) => Conversation;
  selectConversation: (id: string) => void;
  deleteConversation: (id: string) => void;
  addMessage: (conversationId: string, message: Omit<Message, 'id' | 'timestamp'>) => void;
  setLoading: (loading: boolean) => void;
  setStreaming: (streaming: boolean) => void;
  clearMessages: (conversationId: string) => void;
}

const generateId = () => Math.random().toString(36).substring(2, 15);

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  isLoading: false,
  isStreaming: false,

  createConversation: (model: string) => {
    const conversation: Conversation = {
      id: generateId(),
      title: 'New Chat',
      model,
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    
    set((state) => ({
      conversations: [conversation, ...state.conversations],
      activeConversationId: conversation.id,
    }));
    
    return conversation;
  },

  selectConversation: (id: string) => {
    set({ activeConversationId: id });
  },

  deleteConversation: (id: string) => {
    set((state) => {
      const newConversations = state.conversations.filter((c) => c.id !== id);
      return {
        conversations: newConversations,
        activeConversationId: state.activeConversationId === id 
          ? (newConversations[0]?.id || null) 
          : state.activeConversationId,
      };
    });
  },

  addMessage: (conversationId: string, messageData: Omit<Message, 'id' | 'timestamp'>) => {
    const message: Message = {
      ...messageData,
      id: generateId(),
      timestamp: Date.now(),
    };
    
    set((state) => ({
      conversations: state.conversations.map((conv) =>
        conv.id === conversationId
          ? {
              ...conv,
              messages: [...conv.messages, message],
              updatedAt: Date.now(),
              title: conv.messages.length === 0 && messageData.role === 'user'
                ? messageData.content.substring(0, 30) + (messageData.content.length > 30 ? '...' : '')
                : conv.title,
            }
          : conv
      ),
    }));
  },

  setLoading: (loading: boolean) => {
    set({ isLoading: loading });
  },

  setStreaming: (streaming: boolean) => {
    set({ isStreaming: streaming });
  },

  clearMessages: (conversationId: string) => {
    set((state) => ({
      conversations: state.conversations.map((conv) =>
        conv.id === conversationId
          ? { ...conv, messages: [], updatedAt: Date.now() }
          : conv
      ),
    }));
  },
}));
