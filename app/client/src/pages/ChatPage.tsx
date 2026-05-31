import { useState, useRef, useEffect, useCallback } from 'react';
import { Zap, Send, Square, ChevronDown, Sparkles, Plus, Paperclip, Trash2, PanelLeftClose, PanelLeft } from 'lucide-react';
import { useChatStore, type Message } from '../stores/chatStore';
import { useModelState } from '../stores/modelStore';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import type { Model } from '../api/client';
import './ChatPage.css';

export default function ChatPage() {
  const { t } = useI18n();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { requireAuth } = useAuthStore();

  const {
    conversations,
    activeConversationId,
    isStreaming,
    selectedModelId,
    createConversation,
    sendMessage,
    stopStreaming,
    setSelectedModelId,
    setActiveConversation,
    deleteConversation,
  } = useChatStore();

  const { models, loaded, loadModels, refreshModels } = useModelState();

  const [inputValue, setInputValue] = useState('');
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [modelSearch, setModelSearch] = useState('');
  const modelDropdownRef = useRef<HTMLDivElement>(null);
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const activeConv = conversations.find((c) => c.id === activeConversationId) || null;
  const messages = activeConv?.messages || [];
  const activeModelId = selectedModelId || activeConv?.modelId || 'gpt-4o-mini';

  useEffect(() => {
    if (!loaded) loadModels();
  }, [loaded, loadModels]);

  // Refresh models when tab becomes visible (e.g. after editing in admin)
  useEffect(() => {
    const onVisible = () => {
      if (document.visibilityState === 'visible') {
        refreshModels();
      }
    };
    document.addEventListener('visibilitychange', onVisible);
    return () => document.removeEventListener('visibilitychange', onVisible);
  }, [refreshModels]);

  useEffect(() => {
    const mq = window.matchMedia('(max-width: 768px)');
    const handler = (e: MediaQueryListEvent | MediaQueryList) => {
      setSidebarOpen(!e.matches);
    };
    handler(mq);
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, []);

  useEffect(() => {
    if (!modelMenuOpen) return;
    const handler = (e: MouseEvent) => {
      if (modelDropdownRef.current && !modelDropdownRef.current.contains(e.target as Node)) {
        setModelMenuOpen(false);
        setModelSearch('');
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [modelMenuOpen]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  useEffect(() => {
    if (inputRef.current) inputRef.current.focus();
  }, [activeConversationId]);

  const activeModel = models.find((m) => m.id === activeModelId) || null;
  const recentConversations = conversations.slice(0, 30);

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

  const handleNewChat = useCallback(() => {
    const modelId = selectedModelId || 'gpt-4o-mini';
    createConversation(modelId);
    if (window.innerWidth <= 768) setSidebarOpen(false);
  }, [createConversation, selectedModelId]);

  const handleSend = useCallback(() => {
    const trimmed = inputValue.trim();
    const hasFiles = selectedFiles.length > 0;
    if ((!trimmed && !hasFiles) || isStreaming) return;

    if (!requireAuth()) return;

    if (!selectedModelId && !activeConv?.modelId) {
      setSelectedModelId('gpt-4o-mini');
    }

    setInputValue('');
    setSelectedFiles([]);
    sendMessage(trimmed);
  }, [inputValue, selectedFiles, isStreaming, selectedModelId, activeConv, setSelectedModelId, sendMessage, requireAuth]);

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
      if (!requireAuth()) return;
      if (!selectedModelId && !activeConv?.modelId) {
        setSelectedModelId('gpt-4o-mini');
      }
      sendMessage(text);
    },
    [selectedModelId, activeConv, setSelectedModelId, sendMessage, requireAuth]
  );

  const selectModel = useCallback(
    (modelId: string) => {
      setSelectedModelId(modelId);
      setModelMenuOpen(false);
      setModelSearch('');
    },
    [setSelectedModelId]
  );

  const handleFileUpload = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (files.length > 0) {
      setSelectedFiles((prev) => [...prev, ...files]);
    }
    if (fileInputRef.current) fileInputRef.current.value = '';
  }, []);

  const removeFile = useCallback((index: number) => {
    setSelectedFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleSelectConv = useCallback((id: string) => {
    setActiveConversation(id);
    if (window.innerWidth <= 768) setSidebarOpen(false);
  }, [setActiveConversation]);

  const handleDeleteConv = useCallback((e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    deleteConversation(id);
  }, [deleteConversation]);

  const suggestions = [
    t('chat.suggest1'),
    t('chat.suggest2'),
    t('chat.suggest3'),
  ];

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const modelDropdown = (
    <div className="chat-model-dropdown" ref={modelDropdownRef}>
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
  );

  return (
    <div className="chat-page">
      {/* Sidebar */}
      <aside className={`chat-sidebar ${sidebarOpen ? 'open' : ''}`}>
        <div className="chat-sidebar-inner">
          <div className="chat-sidebar-top">
            <span className="chat-sidebar-label">{t('chat.conversations')}</span>
            <button className="chat-sidebar-collapse-btn" onClick={() => setSidebarOpen(false)} title={t('chat.collapseSidebar')}>
              <PanelLeftClose size={15} />
            </button>
          </div>

          <div className="chat-sidebar-convs">
            <div className="chat-sidebar-convs-list">
              {recentConversations.length === 0 ? (
                <div className="chat-sidebar-convs-empty">{t('chat.noConversations')}</div>
              ) : (
                recentConversations.map((conv) => (
                  <div
                    key={conv.id}
                    className={`chat-sidebar-conv-item ${conv.id === activeConversationId ? 'active' : ''}`}
                    onClick={() => handleSelectConv(conv.id)}
                  >
                    <span className="chat-sidebar-conv-title">
                      {conv.title || t('chat.newConversation')}
                    </span>
                    <button
                      className="chat-sidebar-conv-delete"
                      onClick={(e) => handleDeleteConv(e, conv.id)}
                    >
                      <Trash2 size={12} />
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </aside>

      {/* Open sidebar button (visible when collapsed) */}
      {!sidebarOpen && (
        <button className="chat-sidebar-open-btn" onClick={() => setSidebarOpen(true)} title={t('chat.showSidebar')}>
          <PanelLeft size={18} />
        </button>
      )}

      {/* Desktop new chat button */}
      <button className="chat-new-chat-btn" onClick={handleNewChat} title={t('chat.newChat')}>
        <Plus size={18} />
      </button>

      {/* Main area */}
      <div className="chat-main">
        {/* Mobile header bar */}
        <div className="chat-mobile-header">
          <button
            className="chat-mobile-header-btn"
            onClick={() => setSidebarOpen(!sidebarOpen)}
          >
            <PanelLeft size={18} />
          </button>
          <button className="chat-mobile-header-btn" onClick={handleNewChat} title={t('chat.newChat')}>
            <Plus size={18} />
          </button>
        </div>

        {/* Sidebar backdrop (mobile only) */}
        {sidebarOpen && <div className="chat-sidebar-backdrop" onClick={() => setSidebarOpen(false)} />}
        {!activeConv ? (
          <div className="chat-empty">
            <div className="chat-empty-logo">
              <Zap size={32} strokeWidth={1.5} />
            </div>
            <h1 className="chat-empty-title">{t('chat.newChatTitle')}</h1>
            <p className="chat-empty-desc">{t('chat.newChatDesc')}</p>

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
          </div>
        ) : (
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
        )}

        {/* File preview */}
        {selectedFiles.length > 0 && (
          <div className="chat-files-preview">
            {selectedFiles.map((file, i) => (
              <div key={`${file.name}-${i}`} className="chat-file-chip">
                <span className="chat-file-chip-name">{file.name}</span>
                <button className="chat-file-chip-remove" onClick={() => removeFile(i)}>
                  <span>&times;</span>
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Input area */}
        <div className="chat-input-area">
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
            <div className="chat-input-actions">
              <div className="chat-input-actions-left">
                <button
                  className="chat-upload-btn"
                  onClick={handleFileUpload}
                  title={t('chat.uploadFile')}
                >
                  <Paperclip size={15} />
                </button>
                <input
                  ref={fileInputRef}
                  type="file"
                  className="chat-file-input-hidden"
                  onChange={handleFileChange}
                  multiple
                />
                <div className="chat-input-model-selector">
                  <button
                    className="chat-input-model-btn"
                    onClick={() => setModelMenuOpen(!modelMenuOpen)}
                    onMouseDown={(e) => e.stopPropagation()}
                  >
                    <span className="chat-model-dot" />
                    <span className="chat-input-model-name">
                      {activeModel ? activeModel.name : t('chat.selectModel')}
                    </span>
                    <ChevronDown size={11} />
                  </button>
                  {modelMenuOpen && modelDropdown}
                </div>
              </div>
              {isStreaming ? (
                <button className="chat-stop-btn" onClick={stopStreaming}>
                  <Square size={14} />
                </button>
              ) : (
                <button
                  className="chat-send-btn"
                  onClick={handleSend}
                  disabled={(!inputValue.trim() && selectedFiles.length === 0)}
                >
                  <Send size={16} />
                </button>
              )}
            </div>
          </div>
        </div>
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
              <span className="chat-typing-cursor">&#x25CA;</span>
            ) : (
              <div className="chat-markdown">{renderContent(message.content)}</div>
            )}
            {message.isStreaming && message.content && (
              <span className="chat-typing-cursor">&#x25CA;</span>
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
