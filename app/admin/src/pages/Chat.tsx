import { useState, useRef, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import { Send, Square, Trash2, ChevronDown } from 'lucide-react';
import { fetchModels, streamAdminChat, type AdminModel } from '../api/admin';

const providerColors: Record<string, string> = {
  openai: '#34D399',
  anthropic: '#F59E0B',
  google: '#60A5FA',
  deepseek: '#A78BFA',
};

interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
}

export default function Chat() {
  const { t } = useI18n();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const modelDropdownRef = useRef<HTMLDivElement>(null);

  const [models, setModels] = useState<AdminModel[]>([]);
  const [selectedModelId, setSelectedModelId] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [currentStreamContent, setCurrentStreamContent] = useState('');
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const [error, setError] = useState('');

  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    fetchModels()
      .then((res) => {
        const active = (res.data || []).filter((m: AdminModel) => m.is_active);
        setModels(active);
        if (active.length > 0 && !selectedModelId) {
          setSelectedModelId(active[0].name);
        }
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, currentStreamContent]);

  useEffect(() => {
    if (!modelMenuOpen) return;
    const handler = (e: MouseEvent) => {
      if (modelDropdownRef.current && !modelDropdownRef.current.contains(e.target as Node)) {
        setModelMenuOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [modelMenuOpen]);

  const selectedModel = models.find((m) => m.name === selectedModelId || m.model_id === selectedModelId);

  const handleSend = useCallback(async () => {
    const text = inputValue.trim();
    if (!text || !selectedModelId || isStreaming) return;

    setError('');
    setInputValue('');
    setIsStreaming(true);
    setCurrentStreamContent('');

    const userMsg: ChatMessage = { role: 'user', content: text };
    const newMessages = [...messages, userMsg];
    setMessages(newMessages);

    const apiMessages = newMessages.map((m) => ({
      role: m.role,
      content: m.content,
    }));

    let fullContent = '';
    try {
      const gen = streamAdminChat(selectedModelId, apiMessages);
      for await (const chunk of gen) {
        if (chunk.finished) break;
        fullContent += chunk.content;
        setCurrentStreamContent(fullContent);
      }
      setMessages((prev) => [...prev, { role: 'assistant', content: fullContent }]);
      setCurrentStreamContent('');
    } catch (e: any) {
      setError(e.message || t('chat.error'));
    } finally {
      setIsStreaming(false);
    }
  }, [inputValue, selectedModelId, isStreaming, messages, t]);

  const handleStop = () => {
    if (abortRef.current) {
      abortRef.current.abort();
      abortRef.current = null;
    }
    setIsStreaming(false);
    if (currentStreamContent) {
      setMessages((prev) => [...prev, { role: 'assistant', content: currentStreamContent }]);
      setCurrentStreamContent('');
    }
  };

  const handleClear = () => {
    setMessages([]);
    setCurrentStreamContent('');
    setError('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const groupedModels = useCallback(() => {
    const groups: Record<string, AdminModel[]> = {};
    for (const m of models) {
      const key = m.provider_id;
      if (!groups[key]) groups[key] = [];
      groups[key].push(m);
    }
    return groups;
  }, [models]);

  const groups = groupedModels();
  const providerEntries = Object.entries(groups);

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('chat.title')}</h1>
          <p style={styles.pageSubtitle}>{t('chat.subtitle')}</p>
        </div>
        <div style={styles.headerActions}>
          <button style={styles.clearBtn} onClick={handleClear} disabled={messages.length === 0 && !currentStreamContent}>
            <Trash2 size={14} />
            {t('chat.clear')}
          </button>
        </div>
      </header>

      <div style={styles.chatCard}>
        {/* Model Selector */}
        <div style={styles.modelBar}>
          <div style={styles.modelSelector} ref={modelDropdownRef}>
            <button
              style={styles.modelBtn}
              onClick={() => setModelMenuOpen(!modelMenuOpen)}
              disabled={models.length === 0}
            >
              <span style={{ ...styles.modelDot, backgroundColor: selectedModel ? (providerColors[selectedModel.provider_id] || '#A1A1AA') : '#A1A1AA' }} />
              <span style={styles.modelName}>
                {selectedModel ? selectedModel.name : t('chat.selectModel')}
              </span>
              <ChevronDown size={14} style={{ marginLeft: 'auto', opacity: 0.5 }} />
            </button>

            {modelMenuOpen && (
              <div style={styles.modelDropdown}>
                {providerEntries.map(([provider, providerModels]) => (
                  <div key={provider}>
                    <div style={styles.providerGroupLabel}>
                      <span style={{ ...styles.providerDot, backgroundColor: providerColors[provider] || '#A1A1AA' }} />
                      {provider}
                    </div>
                    {providerModels.map((m) => (
                      <button
                        key={m.id}
                        style={{
                          ...styles.modelOption,
                          ...(selectedModelId === m.name ? styles.modelOptionActive : {}),
                        }}
                        onClick={() => {
                          setSelectedModelId(m.name);
                          setModelMenuOpen(false);
                        }}
                      >
                        <span style={styles.modelOptionName}>{m.name}</span>
                        <span style={styles.modelOptionId}>{m.model_id}</span>
                      </button>
                    ))}
                  </div>
                ))}
                {providerEntries.length === 0 && (
                  <div style={styles.emptyText}>{t('chat.noModel')}</div>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Messages Area */}
        <div style={styles.messagesArea}>
          {messages.length === 0 && !currentStreamContent && !error && (
            <div style={styles.welcome}>
              <div style={styles.welcomeIcon}>
                <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                </svg>
              </div>
              <p style={styles.welcomeText}>{t('chat.welcome')}</p>
            </div>
          )}

          {messages.map((msg, i) => (
            <div key={i} style={{
              ...styles.messageRow,
              justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start',
            }}>
              <div style={{
                ...styles.messageBubble,
                backgroundColor: msg.role === 'user' ? '#6366F1' : '#F5F5F4',
                color: msg.role === 'user' ? '#FFFFFF' : '#18181B',
              }}>
                <pre style={styles.messageText}>{msg.content}</pre>
              </div>
            </div>
          ))}

          {(isStreaming || currentStreamContent) && (
            <div style={{ ...styles.messageRow, justifyContent: 'flex-start' }}>
              <div style={{ ...styles.messageBubble, backgroundColor: '#F5F5F4', color: '#18181B' }}>
                <pre style={styles.messageText}>
                  {currentStreamContent || t('chat.thinking')}
                </pre>
                {isStreaming && <span style={styles.cursor}>|</span>}
              </div>
            </div>
          )}

          {error && (
            <div style={{ ...styles.messageRow, justifyContent: 'center' }}>
              <div style={styles.errorBubble}>
                <span>{error}</span>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input Area */}
        <div style={styles.inputArea}>
          <textarea
            ref={inputRef}
            style={styles.input}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t('chat.placeholder')}
            rows={2}
            disabled={!selectedModelId}
          />
          {isStreaming ? (
            <button style={styles.stopBtn} onClick={handleStop} title={t('chat.stop')}>
              <Square size={16} fill="currentColor" />
            </button>
          ) : (
            <button
              style={styles.sendBtn}
              onClick={handleSend}
              disabled={!inputValue.trim() || !selectedModelId}
              title={t('chat.send')}
            >
              <Send size={16} />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: { maxWidth: '900px' },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-end',
    marginBottom: '24px',
  },
  headerActions: {
    display: 'flex',
    gap: '8px',
  },
  pageTitle: {
    fontSize: '24px',
    fontWeight: '700',
    color: '#18181B',
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.02em',
  },
  pageSubtitle: {
    fontSize: '13px',
    color: '#71717A',
    marginTop: '4px',
    fontFamily: "'DM Sans', sans-serif",
  },
  clearBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 14px',
    backgroundColor: '#FFFFFF',
    color: '#71717A',
    border: '1px solid #E7E5E4',
    borderRadius: '10px',
    fontSize: '12px',
    fontWeight: '500',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
  chatCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    display: 'flex',
    flexDirection: 'column',
    height: 'calc(100vh - 160px)',
    minHeight: '500px',
    overflow: 'hidden',
  },
  modelBar: {
    padding: '12px 16px',
    borderBottom: '1px solid #F5F5F4',
  },
  modelSelector: {
    position: 'relative',
    width: 'fit-content',
  },
  modelBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px 14px',
    backgroundColor: '#F5F5F4',
    border: 'none',
    borderRadius: '8px',
    fontSize: '13px',
    fontWeight: '500',
    color: '#18181B',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    minWidth: '200px',
  },
  modelDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    flexShrink: 0,
  },
  modelName: {
    fontSize: '13px',
    fontWeight: '500',
    color: '#18181B',
  },
  modelDropdown: {
    position: 'absolute',
    top: '100%',
    left: 0,
    marginTop: '4px',
    backgroundColor: '#FFFFFF',
    border: '1px solid #E7E5E4',
    borderRadius: '10px',
    boxShadow: '0 4px 16px rgba(0,0,0,0.1)',
    zIndex: 50,
    minWidth: '280px',
    maxHeight: '360px',
    overflowY: 'auto',
    padding: '8px',
  },
  providerGroupLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '11px',
    fontWeight: '500',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    padding: '8px 8px 4px',
    fontFamily: "'DM Sans', sans-serif",
  },
  providerDot: {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
    flexShrink: 0,
  },
  modelOption: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-start',
    width: '100%',
    padding: '8px 10px',
    border: 'none',
    borderRadius: '6px',
    backgroundColor: 'transparent',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    textAlign: 'left',
  },
  modelOptionActive: {
    backgroundColor: '#F5F5F4',
  },
  modelOptionName: {
    fontSize: '13px',
    fontWeight: '500',
    color: '#18181B',
  },
  modelOptionId: {
    fontSize: '11px',
    color: '#A1A1AA',
    marginTop: '1px',
  },
  emptyText: {
    fontSize: '12px',
    color: '#A1A1AA',
    padding: '12px',
    textAlign: 'center',
    fontFamily: "'DM Sans', sans-serif",
  },
  messagesArea: {
    flex: 1,
    overflowY: 'auto',
    padding: '20px',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  messageRow: {
    display: 'flex',
  },
  messageBubble: {
    maxWidth: '75%',
    padding: '10px 14px',
    borderRadius: '12px',
    wordBreak: 'break-word',
  },
  messageText: {
    margin: 0,
    fontSize: '13px',
    lineHeight: '1.6',
    fontFamily: "'DM Sans', sans-serif",
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
  },
  errorBubble: {
    maxWidth: '80%',
    padding: '10px 14px',
    borderRadius: '10px',
    backgroundColor: 'rgba(239, 68, 68, 0.08)',
    color: '#EF4444',
    fontSize: '12px',
    fontFamily: "'DM Sans', sans-serif",
  },
  cursor: {
    animation: 'blink 1s step-end infinite',
    color: '#6366F1',
    fontWeight: '700',
  },
  welcome: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    flex: 1,
    gap: '16px',
  },
  welcomeIcon: {
    width: '64px',
    height: '64px',
    borderRadius: '16px',
    backgroundColor: '#F5F5F4',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  welcomeText: {
    fontSize: '13px',
    color: '#A1A1AA',
    textAlign: 'center',
    maxWidth: '320px',
    fontFamily: "'DM Sans', sans-serif",
  },
  inputArea: {
    display: 'flex',
    gap: '8px',
    padding: '16px',
    borderTop: '1px solid #F5F5F4',
    alignItems: 'flex-end',
  },
  input: {
    flex: 1,
    padding: '10px 14px',
    borderRadius: '10px',
    border: '1px solid #E7E5E4',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
    backgroundColor: '#FFFFFF',
    color: '#18181B',
    outline: 'none',
    resize: 'none',
    maxHeight: '120px',
  },
  sendBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '40px',
    height: '40px',
    borderRadius: '10px',
    border: 'none',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    cursor: 'pointer',
    flexShrink: 0,
  },
  stopBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '40px',
    height: '40px',
    borderRadius: '10px',
    border: 'none',
    backgroundColor: '#EF4444',
    color: '#FFFFFF',
    cursor: 'pointer',
    flexShrink: 0,
  },
};
