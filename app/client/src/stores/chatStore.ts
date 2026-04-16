/**
 * @file Chat Store - 对话状态管理
 * 管理用户的对话历史、当前会话和聊天状态
 * 使用 Zustand 进行状态管理
 */

import { create } from 'zustand';

/**
 * Message - 聊天消息
 * @description 对话中的单条消息，包含角色和内容
 */
export interface Message {
  id: string;               // 消息唯一标识
  role: 'user' | 'assistant' | 'system';  // 消息角色
  content: string;         // 消息内容
  timestamp: number;       // 时间戳
}

/**
 * Conversation - 对话会话
 * @description 包含多条消息的会话，包含标题、模型和消息列表
 */
export interface Conversation {
  id: string;              // 会话唯一标识
  title: string;           // 会话标题（自动从首条用户消息生成）
  model: string;           // 使用的模型 ID
  messages: Message[];    // 消息列表
  createdAt: number;       // 创建时间戳
}

/**
 * ChatState - 聊天状态接口
 * @description 包含聊天相关的所有状态和操作
 */
interface ChatState {
  // State
  conversations: Conversation[];       // 所有会话列表
  activeConversationId: string | null;  // 当前活动会话 ID
  isLoading: boolean;                   // 是否正在等待 AI 响应
  selectedModel: string;               // 当前选择的模型
  showHistory: boolean;                // 是否显示历史记录面板

  // Actions
  setSelectedModel: (model: string) => void;                    // 设置当前模型
  createConversation: (model: string, title?: string) => string; // 创建新会话
  deleteConversation: (id: string) => void;                      // 删除会话
  setActiveConversation: (id: string) => void;                  // 设置当前活动会话
  addMessage: (conversationId: string, message: Omit<Message, 'id' | 'timestamp'>) => void;  // 添加消息
  updateLastAssistantMessage: (conversationId: string, content: string) => void;  // 更新最后一条助手消息（用于流式响应）
  setLoading: (loading: boolean) => void;                        // 设置加载状态
  setShowHistory: (show: boolean) => void;                       // 显示/隐藏历史记录
  getActiveConversation: () => Conversation | undefined;       // 获取当前活动会话
}

// 消息计数器，用于生成唯一消息 ID
let messageCounter = 0;

/**
 * useChatStore - 聊天状态管理 Hook
 * @description 管理对话相关的所有状态
 *
 * 状态说明：
 * - conversations: 保存所有会话，对话数据保存在浏览器本地
 * - activeConversationId: 当前活动的会话 ID
 * - isLoading: 是否正在等待 AI 回复
 * - selectedModel: 当前选中的模型 ID
 * - showHistory: 是否展开历史记录侧边栏
 *
 * 核心功能：
 * - 创建/删除/切换会话
 * - 添加消息到指定会话
 * - 流式响应时更新最后一条助手消息
 */
export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  isLoading: false,
  selectedModel: 'gpt-4o',
  showHistory: false,

  /**
   * setSelectedModel - 设置当前选中的模型
   * @param model - 模型 ID
   */
  setSelectedModel: (model) => set({ selectedModel: model }),

  /**
   * createConversation - 创建新会话
   * @param model - 使用的模型 ID
   * @param title - 可选会话标题，默认"New Chat"
   * @returns 新会话 ID
   * 新会话会添加到列表头部并自动设为活动会话
   */
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

  /**
   * deleteConversation - 删除会话
   * @param id - 会话 ID
   * 如果删除的是当前活动会话，自动切换到下一个或设为 null
   */
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

  /**
   * setActiveConversation - 设置当前活动会话
   * @param id - 会话 ID
   */
  setActiveConversation: (id) => set({ activeConversationId: id }),

  /**
   * addMessage - 添加消息到会话
   * @param conversationId - 会话 ID
   * @param message - 消息内容（不含 id 和 timestamp）
   * 自动生成唯一 ID 和时间戳
   * 如果是会话首条用户消息，自动生成标题（取前40字符）
   */
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

  /**
   * updateLastAssistantMessage - 更新最后一条助手消息
   * @param conversationId - 会话 ID
   * @param content - 新的内容（用于流式响应时逐步更新）
   * 从后向前查找最后一条 assistant 角色消息并更新其内容
   */
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

  /**
   * setLoading - 设置加载状态
   * @param loading - 是否正在加载
   */
  setLoading: (loading) => set({ isLoading: loading }),

  /**
   * setShowHistory - 设置历史记录面板显示状态
   * @param show - 是否显示
   */
  setShowHistory: (show) => set({ showHistory: show }),

  /**
   * getActiveConversation - 获取当前活动会话
   * @returns 当前会话对象或 undefined
   */
  getActiveConversation: () => {
    const state = get();
    return state.conversations.find((c) => c.id === state.activeConversationId);
  },
}));
