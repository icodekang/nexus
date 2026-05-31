import { useState, useRef, useEffect, useCallback, useMemo, memo } from 'react';
import { Zap, Send, Square, ChevronDown, Sparkles, Plus, Paperclip, Trash2, PanelLeftClose, PanelLeft, Copy, Check, AlertTriangle } from 'lucide-react';
import { useChatStore, type Message } from '../stores/chatStore';
import { useModelState } from '../stores/modelStore';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import type { Model } from '../api/client';
import { fetchProviderKeys, type ProviderKeyItem } from '../api/client';
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
  const [userProviderKeys, setUserProviderKeys] = useState<ProviderKeyItem[]>([]);
  const inputHistoryRef = useRef<string[]>([]);
  const historyCursorRef = useRef(-1);

  const activeConv = conversations.find((c) => c.id === activeConversationId) || null;
  const messages = activeConv?.messages || [];
  const activeModelId = selectedModelId || activeConv?.modelId || 'gpt-4o-mini';

  useEffect(() => {
    if (!loaded) loadModels();
  }, [loaded, loadModels]);

  useEffect(() => {
    const token = localStorage.getItem('nexus_token');
    if (token) {
      fetchProviderKeys().then(res => setUserProviderKeys(res.data)).catch(() => {});
    }
  }, [loaded]);

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

  const hasUserKey = activeModel
    ? userProviderKeys.some(
        k => k.provider_slug === activeModel.provider && k.is_active
      )
    : false;
  const modelMissingKeys = activeModel
    ? !activeModel.is_key_configured && !hasUserKey
    : false;

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

    if (trimmed) {
      inputHistoryRef.current = [trimmed, ...inputHistoryRef.current.filter(h => h !== trimmed)].slice(0, 50);
    }
    historyCursorRef.current = -1;
    setInputValue('');
    setSelectedFiles([]);
    sendMessage(trimmed);
  }, [inputValue, selectedFiles, isStreaming, selectedModelId, activeConv, setSelectedModelId, sendMessage, requireAuth]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        const history = inputHistoryRef.current;
        if (history.length === 0) return;
        if (historyCursorRef.current === -1) {
          historyCursorRef.current = 0;
        } else if (historyCursorRef.current < history.length - 1) {
          historyCursorRef.current++;
        }
        setInputValue(history[historyCursorRef.current] || '');
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const history = inputHistoryRef.current;
        if (historyCursorRef.current > 0) {
          historyCursorRef.current--;
          setInputValue(history[historyCursorRef.current] || '');
        } else if (historyCursorRef.current === 0) {
          historyCursorRef.current = -1;
          setInputValue('');
        }
        return;
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
          {modelMissingKeys && (
            <div className="chat-key-warning">
              <AlertTriangle size={14} />
              <span>{t('chat.noKeyWarning')}</span>
            </div>
          )}
          <div className="chat-input-box">
            <textarea
              ref={inputRef}
              className="chat-input"
              rows={1}
              placeholder={t('chat.inputPlaceholder')}
              value={inputValue}
              onChange={(e) => {
                setInputValue(e.target.value);
                historyCursorRef.current = -1;
              }}
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

const MessageBubble = memo(function MessageBubble({
  message,
  formatTime,
}: {
  message: Message;
  formatTime: (ts: number) => string;
}) {
  const isUser = message.role === 'user';
  const rendered = useMemo(
    () => (message.content ? renderContent(message.content) : null),
    [message.content]
  );

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
              <span className="chat-typing-dots">
                <span className="chat-typing-dot" />
                <span className="chat-typing-dot" />
                <span className="chat-typing-dot" />
              </span>
            ) : (
              <div className="chat-markdown">{rendered}</div>
            )}
            {message.isStreaming && message.content && (
              <span className="chat-typing-dots">
                <span className="chat-typing-dot" />
                <span className="chat-typing-dot" />
                <span className="chat-typing-dot" />
              </span>
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
});

function renderContent(text: string): React.ReactNode {
  const elements: React.ReactNode[] = [];
  const lines = text.split('\n');
  let i = 0;
  let key = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (line.trim() === '') {
      elements.push(<br key={key++} />);
      i++;
      continue;
    }

    if (/^#{1,6}\s/.test(line)) {
      const level = line.match(/^(#{1,6})/)![1].length;
      const content = parseInline(line.replace(/^#{1,6}\s*/, ''));
      const Tag = `h${level}` as keyof JSX.IntrinsicElements;
      elements.push(<Tag key={key++}>{content}</Tag>);
      i++;
      continue;
    }

    if (/^[-*]\s/.test(line)) {
      elements.push(
        <ul key={key++}>
          {(() => {
            const items: React.ReactNode[] = [];
            while (i < lines.length && /^[-*]\s/.test(lines[i])) {
              const itemContent = parseInline(lines[i].replace(/^[-*]\s*/, ''));
              items.push(<li key={key++}>{itemContent}</li>);
              i++;
            }
            return items;
          })()}
        </ul>
      );
      continue;
    }

    if (line.startsWith('```')) {
      const lang = line.slice(3).trim().toLowerCase() || 'text';
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].startsWith('```')) {
        codeLines.push(lines[i]);
        i++;
      }
      i++;
      elements.push(
        <CopyCodeBlock key={key++} code={codeLines.join('\n')} lang={lang} />
      );
      continue;
    }

    elements.push(<p key={key++}>{parseInline(line)}</p>);
    i++;
  }

  return <>{elements}</>;
}

function CopyCodeBlock({ code, lang }: { code: string; lang: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      const ta = document.createElement('textarea');
      ta.value = code;
      ta.style.position = 'fixed';
      ta.style.opacity = '0';
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [code]);

  return (
    <div className="chat-code-block">
      <div className="chat-code-header">
        <span className="chat-code-lang">{lang}</span>
        <button className="chat-code-copy-btn" onClick={handleCopy} title="Copy code">
          {copied ? <Check size={13} /> : <Copy size={13} />}
        </button>
      </div>
      <pre>
        <code>{highlightCode(code, lang)}</code>
      </pre>
    </div>
  );
}

type TokenType = 'keyword' | 'string' | 'comment' | 'number' | 'function' | 'builtin' | 'plain';

interface Token {
  text: string;
  type: TokenType;
}

interface LangDef {
  keywords: Set<string>;
  builtins?: Set<string>;
  lineComment?: string;
  blockComment?: [string, string];
}

const LANG_DEFS: Record<string, LangDef> = {
  javascript: {
    keywords: new Set(['const','let','var','function','return','if','else','for','while','do','switch','case','break','continue','new','this','class','extends','import','export','from','default','async','await','try','catch','throw','finally','typeof','instanceof','in','of','void','delete','yield','true','false','null','undefined','static','get','set','super']),
    builtins: new Set(['console','JSON','Math','Promise','Array','Object','String','Number','Boolean','Map','Set','Date','RegExp','Error','parseInt','parseFloat','setTimeout','setInterval','clearTimeout','clearInterval','fetch','document','window','require','module']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  typescript: {
    keywords: new Set(['const','let','var','function','return','if','else','for','while','do','switch','case','break','continue','new','this','class','extends','import','export','from','default','async','await','try','catch','throw','finally','typeof','instanceof','in','of','void','delete','yield','true','false','null','undefined','static','get','set','super','interface','type','enum','implements','abstract','private','public','protected','readonly','as','is','keyof','infer','never','any','unknown','declare','namespace','module']),
    builtins: new Set(['console','JSON','Math','Promise','Array','Object','String','Number','Boolean','Map','Set','Date','RegExp','Error','parseInt','parseFloat','setTimeout','setInterval','clearTimeout','clearInterval','fetch','document','window']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  python: {
    keywords: new Set(['def','return','if','elif','else','for','while','in','not','and','or','is','None','True','False','class','import','from','as','try','except','finally','raise','with','yield','lambda','pass','break','continue','global','nonlocal','assert','del','async','await','self']),
    builtins: new Set(['print','len','range','int','str','float','list','dict','set','tuple','bool','type','isinstance','open','enumerate','zip','map','filter','sorted','reversed','sum','min','max','abs','round','input','super','Exception','ValueError','TypeError','KeyError']),
    lineComment: '#',
  },
  rust: {
    keywords: new Set(['fn','let','mut','const','static','if','else','match','loop','while','for','in','return','break','continue','struct','enum','impl','trait','pub','crate','mod','use','as','self','super','where','type','async','await','move','ref','true','false','Some','None','Ok','Err','unsafe','extern','dyn','box','macro_rules']),
    builtins: new Set(['String','Vec','Option','Result','HashMap','HashSet','println','format','assert','panic','unwrap','expect','clone','into','iter','collect','map','filter','len','push','pop','insert','remove']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  go: {
    keywords: new Set(['func','var','const','type','struct','interface','map','chan','if','else','for','range','switch','case','default','return','break','continue','go','defer','select','package','import','true','false','nil','make','new','append','len','cap','panic','recover']),
    builtins: new Set(['fmt','Println','Printf','Sprintf','Errorf','os','io','http','json','time','strings','strconv','errors','context','sync','net']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  c: {
    keywords: new Set(['auto','break','case','char','const','continue','default','do','double','else','enum','extern','float','for','goto','if','int','long','register','return','short','signed','sizeof','static','struct','switch','typedef','union','unsigned','void','volatile','while','NULL','true','false','inline','restrict']),
    builtins: new Set(['printf','scanf','fprintf','sprintf','snprintf','fopen','fclose','fread','fwrite','fseek','ftell','fgets','malloc','calloc','realloc','free','memcpy','memset','memmove','strlen','strcpy','strcmp','strcat','strncpy','strncmp','exit','abort','assert','size_t','FILE','stdin','stdout','stderr','EOF']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  cpp: {
    keywords: new Set(['auto','break','case','char','const','continue','default','do','double','else','enum','extern','float','for','goto','if','int','long','register','return','short','signed','sizeof','static','struct','switch','typedef','union','unsigned','void','volatile','while','class','public','private','protected','virtual','override','final','template','typename','namespace','using','new','delete','this','try','catch','throw','noexcept','explicit','friend','operator','mutable','constexpr','consteval','constinit','decltype','static_cast','dynamic_cast','reinterpret_cast','const_cast','nullptr','true','false','bool','wchar_t','thread_local','concept','requires','co_await','co_return','co_yield','include','define','ifdef','ifndef','endif','pragma']),
    builtins: new Set(['std','cout','cin','cerr','endl','vector','string','map','unordered_map','set','unordered_set','list','deque','queue','stack','pair','tuple','shared_ptr','unique_ptr','make_shared','make_unique','move','forward','printf','scanf','malloc','free','size_t','iostream','algorithm','numeric','functional','iterator','utility','memory','cstring','cmath']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  java: {
    keywords: new Set(['abstract','assert','boolean','break','byte','case','catch','char','class','const','continue','default','do','double','else','enum','extends','final','finally','float','for','goto','if','implements','import','instanceof','int','interface','long','native','new','package','private','protected','public','return','short','static','strictfp','super','switch','synchronized','this','throw','throws','transient','try','void','volatile','while','true','false','null']),
    builtins: new Set(['System','String','Integer','Boolean','Double','Float','Long','Short','Byte','Object','Class','Math','ArrayList','HashMap','HashSet','List','Map','Set','Stream','Optional','Exception','RuntimeException','IOException','println','print','toString','equals','hashCode']),
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  bash: {
    keywords: new Set(['if','then','else','elif','fi','for','while','do','done','case','esac','in','function','return','exit','export','local','source','declare','readonly','shift','break','continue','echo','cd','ls','rm','mv','cp','mkdir','chmod','chown','grep','sed','awk','cat','head','tail','find','xargs','sort','uniq','wc','tee','curl','wget','ssh','scp','rsync','tar','gzip','gunzip']),
    lineComment: '#',
  },
  sql: {
    keywords: new Set(['SELECT','FROM','WHERE','INSERT','UPDATE','DELETE','CREATE','DROP','ALTER','TABLE','INDEX','INTO','VALUES','SET','AND','OR','NOT','NULL','IS','IN','LIKE','BETWEEN','JOIN','LEFT','RIGHT','INNER','OUTER','ON','AS','ORDER','BY','GROUP','HAVING','LIMIT','OFFSET','UNION','ALL','DISTINCT','COUNT','SUM','AVG','MIN','MAX','EXISTS','CASE','WHEN','THEN','ELSE','END','PRIMARY','KEY','FOREIGN','REFERENCES','CASCADE','DEFAULT','UNIQUE','CHECK','CONSTRAINT','IF','BEGIN','COMMIT','ROLLBACK','TRANSACTION','VARCHAR','INTEGER','BOOLEAN','TEXT','TIMESTAMP','SERIAL','BIGINT','FLOAT']),
    lineComment: '--',
    blockComment: ['/*', '*/'],
  },
  json: {
    keywords: new Set(['true','false','null']),
  },
  css: {
    keywords: new Set(['@media','@import','@keyframes','@font-face','!important']),
    lineComment: undefined,
    blockComment: ['/*', '*/'],
  },
  html: {
    keywords: new Set([]),
    blockComment: ['<!--', '-->'],
  },
  xml: {
    keywords: new Set([]),
    blockComment: ['<!--', '-->'],
  },
};

const LANG_ALIASES: Record<string, string> = {
  js: 'javascript', jsx: 'javascript', ts: 'typescript', tsx: 'typescript',
  py: 'python', py3: 'python',
  rs: 'rust',
  sh: 'bash', shell: 'bash', zsh: 'bash',
  yaml: 'yaml', yml: 'yaml',
  md: 'markdown', markdown: 'markdown',
  txt: 'text', text: 'text', plain: 'text',
  c: 'c', cpp: 'cpp', cxx: 'cpp', 'c++': 'cpp', h: 'c',
  java: 'java', kt: 'kotlin', kotlin: 'kotlin',
  rb: 'ruby', ruby: 'ruby',
  php: 'php',
  swift: 'swift',
  scala: 'scala',
  lua: 'lua',
  r: 'r',
  perl: 'perl',
  dart: 'dart',
  toml: 'toml', ini: 'ini', cfg: 'ini',
  dockerfile: 'dockerfile', docker: 'dockerfile',
  makefile: 'makefile', make: 'makefile',
  graphql: 'graphql', gql: 'graphql',
  proto: 'protobuf', protobuf: 'protobuf',
};

function getLangDef(lang: string): LangDef | null {
  const resolved = LANG_ALIASES[lang] || lang;
  return LANG_DEFS[resolved] || null;
}

function highlightCode(code: string, lang: string): React.ReactNode[] {
  const def = getLangDef(lang);
  const lines = code.split('\n');
  const result: React.ReactNode[] = [];
  let key = 0;

  for (let li = 0; li < lines.length; li++) {
    if (li > 0) result.push('\n');
    const line = lines[li];

    if (!def) {
      result.push(<span key={key++}>{line}</span>);
      continue;
    }

    const tokens = tokenizeLine(line, def);
    for (const t of tokens) {
      result.push(
        <span key={key++} className={`code-token code-${t.type}`}>
          {t.text}
        </span>
      );
    }
  }

  return result;
}

function tokenizeLine(line: string, def: LangDef): Token[] {
  const tokens: Token[] = [];
  let i = 0;

  while (i < line.length) {
    // Check for line comment
    if (def.lineComment && line.startsWith(def.lineComment, i)) {
      tokens.push({ text: line.slice(i), type: 'comment' });
      return tokens;
    }

    // Check for block comment start
    if (def.blockComment) {
      const [open, close] = def.blockComment;
      if (line.startsWith(open, i)) {
        const end = line.indexOf(close, i + open.length);
        if (end !== -1) {
          tokens.push({ text: line.slice(i, end + close.length), type: 'comment' });
          i = end + close.length;
          continue;
        } else {
          tokens.push({ text: line.slice(i), type: 'comment' });
          return tokens;
        }
      }
    }

    // String literals
    if (line[i] === '"' || line[i] === "'" || line[i] === '`') {
      const quote = line[i];
      let j = i + 1;
      while (j < line.length) {
        if (line[j] === '\\') { j += 2; continue; }
        if (line[j] === quote) { j++; break; }
        j++;
      }
      tokens.push({ text: line.slice(i, j), type: 'string' });
      i = j;
      continue;
    }

    // Numbers
    if (/[\d]/.test(line[i]) && (i === 0 || /[\s({[\],;:=+\-*/%&|^<>!?~]/.test(line[i - 1]))) {
      let j = i;
      while (j < line.length && /[\d.xXa-fA-F_ep]/.test(line[j])) j++;
      const num = line.slice(i, j);
      if (/^0x[\da-fA-F_]+$/.test(num) || /^\d[\d_]*(\.[\d_]+)?([eE][+\-]?[\d_]+)?$/.test(num)) {
        tokens.push({ text: num, type: 'number' });
        i = j;
        continue;
      }
    }

    // Word tokens (keywords, builtins, identifiers)
    if (/[a-zA-Z_$@]/.test(line[i])) {
      let j = i;
      while (j < line.length && /[\w$]/.test(line[j])) j++;
      const word = line.slice(i, j);

      if (def.keywords.has(word)) {
        tokens.push({ text: word, type: 'keyword' });
      } else if (def.builtins?.has(word)) {
        tokens.push({ text: word, type: 'builtin' });
      } else if (j < line.length && line[j] === '(') {
        tokens.push({ text: word, type: 'function' });
      } else {
        tokens.push({ text: word, type: 'plain' });
      }
      i = j;
      continue;
    }

    // Fallthrough: single character
    tokens.push({ text: line[i], type: 'plain' });
    i++;
  }

  return tokens;
}

function parseInline(text: string): React.ReactNode[] {
  const parts: React.ReactNode[] = [];
  let key = 0;

  const regex = /(\*\*(.+?)\*\*|\*(.+?)\*|`(.+?)`)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    if (match[2] !== undefined) {
      parts.push(<strong key={key++}>{match[2]}</strong>);
    } else if (match[3] !== undefined) {
      parts.push(<em key={key++}>{match[3]}</em>);
    } else if (match[4] !== undefined) {
      parts.push(<code key={key++}>{match[4]}</code>);
    }
    lastIndex = regex.lastIndex;
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return parts;
}
