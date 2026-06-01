import { create } from 'zustand';
import { streamChat, type ChatMessage } from '../api/client';

const CONVERSATIONS_KEY = 'nexus_conversations';
const ACTIVE_ID_KEY = 'nexus_active_conversation';
const SELECTED_MODEL_KEY = 'nexus_selected_model';

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  model?: string;
  timestamp: number;
  isStreaming?: boolean;
}

export interface Conversation {
  id: string;
  title: string;
  modelId: string;
  messages: Message[];
  createdAt: number;
  updatedAt: number;
}

function loadConversations(): Conversation[] {
  try {
    const raw = localStorage.getItem(CONVERSATIONS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveConversations(convs: Conversation[]) {
  localStorage.setItem(CONVERSATIONS_KEY, JSON.stringify(convs));
}

function loadActiveId(): string | null {
  return localStorage.getItem(ACTIVE_ID_KEY);
}

function saveActiveId(id: string | null) {
  if (id) {
    localStorage.setItem(ACTIVE_ID_KEY, id);
  } else {
    localStorage.removeItem(ACTIVE_ID_KEY);
  }
}

function loadSelectedModelId(): string | null {
  return localStorage.getItem(SELECTED_MODEL_KEY);
}

function saveSelectedModelId(id: string | null) {
  if (id) {
    localStorage.setItem(SELECTED_MODEL_KEY, id);
  } else {
    localStorage.removeItem(SELECTED_MODEL_KEY);
  }
}

let idCounter = Date.now();
function genId(): string {
  return (++idCounter).toString(36);
}

interface ChatState {
  conversations: Conversation[];
  activeConversationId: string | null;
  isStreaming: boolean;
  selectedModelId: string | null;

  createConversation: (modelId: string) => string;
  deleteConversation: (id: string) => void;
  setActiveConversation: (id: string | null) => void;
  setSelectedModelId: (id: string | null) => void;
  sendMessage: (content: string) => Promise<void>;
  stopStreaming: () => void;
  renameConversation: (id: string, title: string) => void;
}

let abortController: AbortController | null = null;
let currentGenerator: AsyncGenerator<string, void, unknown> | null = null;

export const useChatStore = create<ChatState>((set, get) => {
  const persisted = loadConversations();
  const activeId = loadActiveId();

  return {
    conversations: persisted,
    activeConversationId: persisted.length > 0 ? activeId : null,
    isStreaming: false,
    selectedModelId: loadSelectedModelId(),

    createConversation: (modelId) => {
      const id = genId();
      const conv: Conversation = {
        id,
        title: '',
        modelId,
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      set((s) => {
        const convs = [conv, ...s.conversations];
        saveConversations(convs);
        saveActiveId(id);
        saveSelectedModelId(modelId);
        return { conversations: convs, activeConversationId: id, selectedModelId: modelId };
      });
      return id;
    },

    deleteConversation: (id) => {
      set((s) => {
        const convs = s.conversations.filter((c) => c.id !== id);
        saveConversations(convs);
        const newActive = s.activeConversationId === id
          ? (convs[0]?.id ?? null)
          : s.activeConversationId;
        saveActiveId(newActive);
        return { conversations: convs, activeConversationId: newActive };
      });
    },

    setActiveConversation: (id) => {
      saveActiveId(id);
      set({ activeConversationId: id });
    },

    setSelectedModelId: (id) => {
      saveSelectedModelId(id);
      set({ selectedModelId: id });
    },

    sendMessage: async (content) => {
      const state = get();
      let convId = state.activeConversationId;
      let modelId = state.selectedModelId || 'gpt-4o-mini';

      if (!convId) {
        convId = get().createConversation(modelId);
      }

      const conv = get().conversations.find((c) => c.id === convId);
      if (!conv) return;

      if (!modelId) {
        modelId = conv.modelId;
      }

      const userMsg: Message = {
        id: genId(),
        role: 'user',
        content,
        timestamp: Date.now(),
      };

      const assistantMsg: Message = {
        id: genId(),
        role: 'assistant',
        content: '',
        model: modelId,
        timestamp: Date.now(),
        isStreaming: true,
      };

      set((s) => {
        const convs = s.conversations.map((c) => {
          if (c.id !== convId) return c;
          const title = c.messages.length === 0 ? content.slice(0, 50) : c.title;
          return {
            ...c,
            title,
            modelId,
            messages: [...c.messages, userMsg, assistantMsg],
            updatedAt: Date.now(),
          };
        });
        saveConversations(convs);
        return { conversations: convs, isStreaming: true };
      });

      const messages: ChatMessage[] = [
        ...conv.messages.map((m) => ({ role: m.role, content: m.content } as ChatMessage)),
        { role: 'user', content },
      ];

      try {
        abortController = new AbortController();
        const generator = streamChat(modelId, messages, convId);
        currentGenerator = generator;
        let fullContent = '';
        let rafId: number | null = null;
        const convRef = { current: convId };
        const msgRef = { current: assistantMsg.id };

        const flush = () => {
          rafId = null;
          const content = fullContent;
          set((s) => {
            const convs = s.conversations.map((c) => {
              if (c.id !== convRef.current) return c;
              return {
                ...c,
                messages: c.messages.map((m) =>
                  m.id === msgRef.current
                    ? { ...m, content, model: modelId }
                    : m
                ),
                updatedAt: Date.now(),
              };
            });
            return { conversations: convs };
          });
        };

        for await (const chunk of generator) {
          fullContent += chunk;
          if (rafId === null) {
            rafId = requestAnimationFrame(() => flush());
          }
        }

        if (rafId !== null) {
          cancelAnimationFrame(rafId);
        }
        flush();
      } catch {
        set((s) => {
          const convs = s.conversations.map((c) => {
            if (c.id !== convId) return c;
            return {
              ...c,
              messages: c.messages.map((m) =>
                m.id === assistantMsg.id
                  ? { ...m, content: m.content || 'Request failed. Please try again.', isStreaming: false }
                  : m
              ),
              updatedAt: Date.now(),
            };
          });
          saveConversations(convs);
          return { conversations: convs, isStreaming: false };
        });
        return;
      }

      set((s) => {
        const convs = s.conversations.map((c) => {
          if (c.id !== convId) return c;
          return {
            ...c,
            messages: c.messages.map((m) =>
              m.id === assistantMsg.id ? { ...m, isStreaming: false } : m
            ),
            updatedAt: Date.now(),
          };
        });
        saveConversations(convs);
        return { conversations: convs, isStreaming: false };
      });
    },

    stopStreaming: () => {
      if (abortController) {
        abortController.abort();
        abortController = null;
      }
      if (currentGenerator) {
        currentGenerator.return?.();
        currentGenerator = null;
      }
      set((s) => {
        const convs = s.conversations.map((c) => {
          if (c.id !== s.activeConversationId) return c;
          return {
            ...c,
            messages: c.messages.map((m) =>
              m.isStreaming ? { ...m, isStreaming: false } : m
            ),
          };
        });
        saveConversations(convs);
        return { conversations: convs, isStreaming: false };
      });
    },

    renameConversation: (id, title) => {
      set((s) => {
        const convs = s.conversations.map((c) =>
          c.id === id ? { ...c, title } : c
        );
        saveConversations(convs);
        return { conversations: convs };
      });
    },
  };
});
