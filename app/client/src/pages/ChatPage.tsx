import { useState, useRef, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Zap, Send, Square, ChevronDown, Sparkles } from 'lucide-react';
import { useChatStore, type Message, type Conversation } from '../stores/chatStore';
import { useModelState } from '../stores/modelStore';
import { useI18n } from '../i18n';
import type { Model } from '../api/client';
import './ChatPage.css';

function ChatPage() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const {
    conversations,
    activeConversationId,
    isStreaming,
    selectedModelId,
    createConversation,
    sendMessage,
    stopStreaming,
    setSelectedModelId,
  } = useChatStore();

  const { models, loaded, loadModels } = useModelState();

  const [inputValue, setInputValue] = useState('');
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [modelSearch, setModelSearch] = useState('');

  const activeConv = conversations.find((c) => c.id === activeConversationId) || null;
  const messages = activeConv?.messages || [];
  const activeModelId = selectedModelId || activeConv?.modelId || 'gpt-4o-mini';

  useEffect(() => {
    if (!loaded) loadModels();
  }, [loaded, loadModels]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  useEffect(() => {
    if (inputRef.current) inputRef.current.focus();
  }, [activeConversationId]);

  const activeModel = models.find((m) => m.id === activeModelId) || null;

  const groupedModels = useCallback(() => {
    const groups: Record<string, Model[]> = {};
    for (const m of models) {
      const key = m.provider_name || m.provider;
      if (!groups[key]) groups[key] = [];
      groups[key].push(m);
    }
    return groups;
  }, [models]);

  const filteredGrouped = useCallback(() => {
    if (!modelSearch) return groupedModels();
    const q = modelSearch.toLowerCase();
    const result: Record<string, Model[]> = {};
    for (const [provider, list] of Object.entries(groupedModels())) {
      const filtered = list.filter(
        (m) => m.name.toLowerCase().includes(q) || m.id.toLowerCase().includes(q)
      );
      if (filtered.length > 0) result[provider] = filtered;
    }
    return result;
  }, [groupedModels, modelSearch]);

  const handleSend = useCallback(() => {
    const trimmed = inputValue.trim();
    if (!trimmed || isStreaming) return;

    if (!selectedModelId && !activeConv?.modelId) {
      setSelectedModelId('gpt-4o-mini');
    }

    setInputValue('');
    sendMessage(trimmed);
  }, [inputValue, isStreaming, selectedModelId, activeConv, setSelectedModelId, sendMessage]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const handleSuggestion = useCallback(
    (text: string) => {
      if (!selectedModelId && !activeConv?.modelId) {
        setSelectedModelId('gpt-4o-mini');
      }
      sendMessage(text);
    },
    [selectedModelId, activeConv, setSelectedModelId, sendMessage]
  );

  const selectModel = useCallback(
    (modelId: string) => {
      setSelectedModelId(modelId);
      setModelMenuOpen(false);
      setModelSearch('');
    },
    [setSelectedModelId]
  );

  const suggestions = [
    t('chat.suggest1'),
    t('chat.suggest2'),
    t('chat.suggest3'),
  ];

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="chat-page">
      <div className="chat-content">
        {!activeConv ? (
          <div className="chat-empty">
            <div className="chat-empty-logo">
              <Zap size={32} strokeWidth={1.5} />
            </div>
            <h1 className="chat-empty-title">{t('chat.newChatTitle')}</h1>
            <p className="chat-empty-desc">
              {t('chat.newChatDesc')}
            </p>

            <div className="chat-model-selector">
              <button
                className="chat-model-btn"
                onClick={() => setModelMenuOpen(!modelMenuOpen)}
              >
                {activeModel ? activeModel.name : t('chat.selectModel')}
                <ChevronDown size={14} />
              </button>
              {modelMenuOpen && (
                <div className="chat-model-dropdown">
                  <div className="chat-model-search-wrapper">
                    <input
                      className="chat-model-search"
                      type="text"
                      placeholder={t('chat.searchModels')}
                      value={modelSearch}
                      onChange={(e) => setModelSearch(e.target.value)}
                      autoFocus
                    />
                  </div>
                  <div className="chat-model-list">
                    {Object.entries(filteredGrouped()).length === 0 && (
                      <div className="chat-model-empty">{t('chat.noModelsFound')}</div>
                    )}
                    {Object.entries(filteredGrouped()).map(([provider, list]) => (
                      <div key={provider} className="chat-model-group">
                        <div className="chat-model-group-label">{provider}</div>
                        {list.map((m) => (
                          <button
                            key={m.id}
                            className={`chat-model-item ${m.id === activeModelId ? 'active' : ''}`}
                            onClick={() => selectModel(m.id)}
                          >
                            <div className="chat-model-item-name">{m.name}</div>
                            <div className="chat-model-item-meta">
                              <span className="chat-model-item-id">{m.id}</span>
                              {m.context_window > 0 && (
                                <span className="chat-model-item-ctx">
                                  {Math.round(m.context_window / 1024)}K
                                </span>
                              )}
                            </div>
                          </button>
                        ))}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="chat-suggestions">
              {suggestions.map((s, i) => (
                <button
                  key={i}
                  className="chat-suggestion-card"
                  onClick={() => handleSuggestion(s)}
                >
                  <span>{s}</span>
                </button>
              ))}
            </div>

            <div className="chat-input-wrapper chat-input-centered">
              <div className="chat-input-box">
                <textarea
                  ref={inputRef}
                  className="chat-input"
                  rows={1}
                  placeholder={t('chat.inputPlaceholder')}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={handleKeyDown}
                  disabled={isStreaming}
                />
                <button
                  className="chat-send-btn"
                  onClick={handleSend}
                  disabled={!inputValue.trim() || isStreaming}
                >
                  <Send size={16} />
                </button>
              </div>
              <p className="chat-disclaimer">{t('chat.disclaimer')}</p>
            </div>
          </div>
        ) : (
          <>
            <div className="chat-header">
              <button
                className="chat-model-btn"
                onClick={() => setModelMenuOpen(!modelMenuOpen)}
              >
                {activeModel ? (
                  <>
                    <span className="chat-model-dot" />
                    {activeModel.name}
                  </>
                ) : (
                  t('chat.selectModel')
                )}
                <ChevronDown size={14} />
              </button>
              {modelMenuOpen && (
                <div className="chat-model-dropdown">
                  <div className="chat-model-search-wrapper">
                    <input
                      className="chat-model-search"
                      type="text"
                      placeholder={t('chat.searchModels')}
                      value={modelSearch}
                      onChange={(e) => setModelSearch(e.target.value)}
                      autoFocus
                    />
                  </div>
                  <div className="chat-model-list">
                    {Object.entries(filteredGrouped()).length === 0 && (
                      <div className="chat-model-empty">{t('chat.noModelsFound')}</div>
                    )}
                    {Object.entries(filteredGrouped()).map(([provider, list]) => (
                      <div key={provider} className="chat-model-group">
                        <div className="chat-model-group-label">{provider}</div>
                        {list.map((m) => (
                          <button
                            key={m.id}
                            className={`chat-model-item ${m.id === activeModelId ? 'active' : ''}`}
                            onClick={() => selectModel(m.id)}
                          >
                            <div className="chat-model-item-name">{m.name}</div>
                            <div className="chat-model-item-meta">
                              <span className="chat-model-item-id">{m.id}</span>
                              {m.context_window > 0 && (
                                <span className="chat-model-item-ctx">
                                  {Math.round(m.context_window / 1024)}K
                                </span>
                              )}
                            </div>
                          </button>
                        ))}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="chat-messages">
              {messages
                .filter((m) => m.role !== 'system')
                .map((m) => (
                  <MessageBubble
                    key={m.id}
                    message={m}
                    formatTime={formatTime}
                  />
                ))}
              <div ref={messagesEndRef} />
            </div>

            <div className="chat-input-wrapper chat-input-bottom">
              <div className="chat-input-box">
                <textarea
                  ref={inputRef}
                  className="chat-input"
                  rows={1}
                  placeholder={t('chat.inputPlaceholder')}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={handleKeyDown}
                  disabled={isStreaming}
                />
                {isStreaming ? (
                  <button
                    className="chat-stop-btn"
                    onClick={stopStreaming}
                  >
                    <Square size={14} />
                  </button>
                ) : (
                  <button
                    className="chat-send-btn"
                    onClick={handleSend}
                    disabled={!inputValue.trim()}
                  >
                    <Send size={16} />
                  </button>
                )}
              </div>
              <p className="chat-disclaimer">{t('chat.disclaimer')}</p>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function MessageBubble({
  message,
  formatTime,
}: {
  message: Message;
  formatTime: (ts: number) => string;
}) {
  const isUser = message.role === 'user';

  return (
    <div className={`chat-message ${isUser ? 'user' : 'assistant'}`}>
      <div className="chat-message-inner">
        <div className="chat-message-avatar">
          {isUser ? 'U' : <Sparkles size={14} />}
        </div>
        <div className="chat-message-body">
          <div className="chat-message-content">
            {isUser ? (
              <p>{message.content}</p>
            ) : message.isStreaming && !message.content ? (
              <span className="chat-typing-cursor">▊</span>
            ) : (
              <div className="chat-markdown">{renderContent(message.content)}</div>
            )}
            {message.isStreaming && message.content && (
              <span className="chat-typing-cursor">▊</span>
            )}
          </div>
          <div className="chat-message-meta">
            {message.model && (
              <span className="chat-message-model">{message.model}</span>
            )}
            <span className="chat-message-time">{formatTime(message.timestamp)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function renderContent(content: string): React.ReactNode {
  const lines = content.split('\n');
  return (
    <>
      {lines.map((line, i) => (
        <span key={i}>
          {i > 0 && <br />}
          {line}
        </span>
      ))}
    </>
  );
}

export default ChatPage;
