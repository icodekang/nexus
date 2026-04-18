/**
 * @file ChatPage - AI 对话聊天页面
 * 支持流式和非流式对话，会话历史管理，模型选择
 * 使用 chatStore 管理对话状态，modelStore 管理可用模型
 */
import { useState, useRef, useEffect } from 'react';
import { Send, Plus, Trash2, ChevronDown, Sparkles, Check, Search, History } from 'lucide-react';
import { useChatStore } from '../stores/chatStore';
import { useModelState } from '../stores/modelStore';
import { sendChat, streamChat } from '../api/client';
import { useI18n } from '../i18n';
import { getErrorMessage } from '../utils/errors';
import './ChatPage.css';

// 提供商元信息：颜色和显示名称
const PROVIDER_META: Record<string, { color: string; label: string }> = {
  openai: { color: '#10B981', label: 'OpenAI' },
  anthropic: { color: '#D97706', label: 'Anthropic' },
  google: { color: '#3B82F6', label: 'Google' },
  deepseek: { color: '#8B5CF6', label: 'DeepSeek' },
};

/**
 * ChatPage - 聊天主组件
 * @description 管理对话、发送消息、选择模型、处理流式响应
 */
export default function ChatPage() {
  const {
    conversations, activeConversationId, isLoading, selectedModel, showHistory,
    setSelectedModel, createConversation, deleteConversation,
    setActiveConversation, addMessage, setLoading, setShowHistory,
  } = useChatStore();
  const { models, getModelsByProvider, loadModels, loaded } = useModelState();
  const { t } = useI18n();
  const [input, setInput] = useState('');
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [modelSearch, setModelSearch] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const modelPickerRef = useRef<HTMLDivElement>(null);

  // 获取当前活动的会话
  const activeConversation = conversations.find((c) => c.id === activeConversationId);

  // 加载可用模型
  useEffect(() => {
    if (!loaded) loadModels();
  }, [loaded, loadModels]);

  // 收到新消息时自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [activeConversation?.messages.length]);

  // 点击模型选择器外部时关闭选择器
  useEffect(() => {
    if (!showModelPicker) return;
    const handler = (e: MouseEvent) => {
      if (modelPickerRef.current && !modelPickerRef.current.contains(e.target as Node)) {
        setShowModelPicker(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [showModelPicker]);

  // 发送消息：创建/追加会话，调用流式或非流式 API
  const handleSend = async () => {
    const text = input.trim();
    if (!text || isLoading) return;

    let convId = activeConversationId;
    if (!convId) {
      convId = createConversation(selectedModel, t('chat.newChat'));
    }

    addMessage(convId!, { role: 'user', content: text });
    addMessage(convId!, { role: 'assistant', content: '' });
    setInput('');
    setLoading(true);

    try {
      const conv = useChatStore.getState().conversations.find((c) => c.id === convId);
      // 获取历史消息（排除空的 assistant 消息），作为上下文发送
      const messages = (conv?.messages || [])
        .filter((m) => m.role !== 'assistant' || m.content !== '')
        .map((m) => ({ role: m.role, content: m.content }));

      messages.push({ role: 'user', content: text });

      // 使用流式 API 发送消息，通过 x-session-id 实现密钥亲和性
      let fullContent = '';
      const stream = streamChat(selectedModel, messages, convId);
      for await (const chunk of stream) {
        fullContent += chunk;
        // 实时更新最后一条 assistant 消息的内容
        useChatStore.getState().updateLastAssistantMessage(convId!, fullContent);
      }

      // 如果流式响应无内容，降级到非流式 API
      if (!fullContent) {
        const resp = await sendChat(selectedModel, messages, convId);
        const assistantContent = resp.choices[0]?.message?.content || t('chat.noResponse');
        useChatStore.getState().updateLastAssistantMessage(convId!, assistantContent);
      }
    } catch (err: unknown) {
      const message = getErrorMessage(err, t);
      useChatStore.getState().updateLastAssistantMessage(
        convId!,
        t('chat.errorPrefix', { message })
      );
    } finally {
      setLoading(false);
    }
  };

  // 处理键盘事件：Enter 发送消息（Shift+Enter 换行）
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // 创建新对话
  const handleNewChat = () => {
    createConversation(selectedModel, t('chat.newChat'));
    setShowHistory(false);
    inputRef.current?.focus();
  };

  // 获取当前选中的模型信息
  const selectedModelData = models.find((m) => m.id === selectedModel);
  const modelDisplayName = selectedModelData?.name || selectedModel;
  const selectedProviderMeta = selectedModelData ? PROVIDER_META[selectedModelData.provider] : null;

  // 按提供商分组并支持搜索过滤模型
  const grouped = getModelsByProvider();
  const filteredGrouped: Record<string, typeof models> = {};
  const search = modelSearch.toLowerCase();
  for (const [provider, providerModels] of Object.entries(grouped)) {
    const filtered = providerModels.filter((m) =>
      !search || m.name.toLowerCase().includes(search) || m.id.toLowerCase().includes(search) || provider.toLowerCase().includes(search)
    );
    if (filtered.length > 0) filteredGrouped[provider] = filtered;
  }

  return (
    <div className="chat-page">
      {/* Chat header */}
      <header className="chat-header">
        <div className="chat-header-left">
          <button
            className="chat-history-toggle"
            onClick={() => setShowHistory(!showHistory)}
          >
            <History size={16} />
            <span>{t('chat.history')}{conversations.length > 0 ? ` (${conversations.length})` : ''}</span>
          </button>
        </div>

        <div className="chat-header-right">
          <button className="chat-new-btn" onClick={handleNewChat}>
            <Plus size={16} />
            <span>{t('chat.newChat')}</span>
          </button>
        </div>
      </header>

      <div className="chat-body">
        {/* History sidebar (mobile overlay) */}
        {showHistory && (
          <div className="chat-history-overlay" onClick={() => setShowHistory(false)}>
            <div className="chat-history-panel" onClick={(e) => e.stopPropagation()}>
              <h3 className="chat-history-title">{t('chat.conversations')}</h3>
              <div className="chat-history-list">
                {conversations.length === 0 ? (
                  <div className="chat-history-empty">{t('chat.noConversations')}</div>
                ) : (
                  conversations.map((c) => (
                    <div
                      key={c.id}
                      className={`chat-history-item ${c.id === activeConversationId ? 'active' : ''}`}
                      onClick={() => {
                        setActiveConversation(c.id);
                        setShowHistory(false);
                      }}
                    >
                      <span className="chat-history-item-title">{c.title}</span>
                      <button
                        className="chat-history-delete"
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteConversation(c.id);
                        }}
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        )}

        {/* Messages area */}
        <div className="chat-messages">
          {!activeConversation || activeConversation.messages.length === 0 ? (
            <div className="chat-empty">
              <div className="chat-empty-icon">
                <Sparkles size={28} strokeWidth={1.5} />
              </div>
              <h2 className="chat-empty-title">{t('chat.startConversation')}</h2>
              <p className="chat-empty-desc">
                {t('chat.startDesc')}
              </p>
              <div className="chat-suggestions">
                {[t('chat.suggest1'), t('chat.suggest2'), t('chat.suggest3')].map((s) => (
                  <button
                    key={s}
                    className="chat-suggestion"
                    onClick={() => {
                      setInput(s);
                      inputRef.current?.focus();
                    }}
                  >
                    {s}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <>
              {activeConversation.messages.map((msg) => (
                <div key={msg.id} className={`chat-bubble-row ${msg.role}`}>
                  {msg.role === 'assistant' && (
                    <div className="chat-bubble-avatar assistant-avatar">
                      <Sparkles size={12} />
                    </div>
                  )}
                  <div className={`chat-bubble ${msg.role}`}>
                    {msg.content || (msg.role === 'assistant' && isLoading ? (
                      <span className="chat-typing">
                        <span /><span /><span />
                      </span>
                    ) : null)}
                  </div>
                  {msg.role === 'user' && (
                    <div className="chat-bubble-avatar user-avatar">
                      U
                    </div>
                  )}
                </div>
              ))}
              <div ref={messagesEndRef} />
            </>
          )}
        </div>
      </div>

      {/* Input area */}
      <div className="chat-input-area">
        <div className="chat-input-wrapper" ref={modelPickerRef}>
          {/* Model selector button inside the input bar — right side, below textarea */}
          <div className="chat-input-model-btn-wrapper">
            <button
              className={`chat-model-selector ${showModelPicker ? 'open' : ''}`}
              onClick={() => setShowModelPicker(!showModelPicker)}
            >
              {selectedProviderMeta && (
                <span className="chat-model-dot" style={{ background: selectedProviderMeta.color }} />
              )}
              <span className="chat-model-selector-name">{modelDisplayName}</span>
              <ChevronDown size={12} className={`chat-model-chevron ${showModelPicker ? 'rotated' : ''}`} />
            </button>
          </div>

          <textarea
            ref={inputRef}
            className="chat-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('chat.inputPlaceholder')}
            rows={1}
          />
          <button
            className={`chat-send-btn ${input.trim() ? 'ready' : ''}`}
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
          >
            <Send size={16} />
          </button>

          {/* Model dropdown — opens upward from input bar */}
          {showModelPicker && (
            <div className="chat-model-dropdown">
              <div className="chat-model-search-wrapper">
                <Search size={14} className="chat-model-search-icon" />
                <input
                  className="chat-model-search"
                  placeholder={t('chat.searchModels')}
                  value={modelSearch}
                  onChange={(e) => setModelSearch(e.target.value)}
                  autoFocus
                />
              </div>
              <div className="chat-model-list">
                {Object.entries(filteredGrouped).map(([provider, providerModels]) => {
                  const meta = PROVIDER_META[provider] || { color: '#A8A29E', label: provider };
                  return (
                    <div key={provider} className="chat-model-group">
                      <div className="chat-model-group-header">
                        <span className="chat-model-group-dot" style={{ background: meta.color }} />
                        <span>{meta.label}</span>
                      </div>
                      {providerModels.map((m) => (
                        <button
                          key={m.id}
                          className={`chat-model-option ${m.id === selectedModel ? 'selected' : ''}`}
                          onClick={() => {
                            setSelectedModel(m.id);
                            setShowModelPicker(false);
                            setModelSearch('');
                          }}
                        >
                          <div className="chat-model-option-info">
                            <span className="chat-model-option-name">{m.name}</span>
                            <span className="chat-model-option-context">{t('chat.contextWindow', { size: (m.context_window / 1000).toFixed(0) })}</span>
                          </div>
                          {m.id === selectedModel && <Check size={14} className="chat-model-option-check" />}
                        </button>
                      ))}
                    </div>
                  );
                })}
                {Object.keys(filteredGrouped).length === 0 && (
                  <div className="chat-model-empty">{t('chat.noModelsFound')}</div>
                )}
              </div>
            </div>
          )}
        </div>
        <p className="chat-disclaimer">
          {t('chat.disclaimer')}
        </p>
      </div>
    </div>
  );
}
