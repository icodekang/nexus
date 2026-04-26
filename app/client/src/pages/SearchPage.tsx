/**
 * @file SearchPage - 多模型搜索对比页面
 * 编辑风格的搜索界面：同时查询多个 AI 模型，按评分排序展示
 * 桌面端采用双栏布局（最佳回答 + 其余排名列表）
 */

import { useState, useRef, useEffect } from 'react';
import { useChatStore } from '../stores/chatStore';
import { fetchBatchChat, type ModelResult } from '../api/client';
import { useI18n } from '../i18n';
import './SearchPage.css';

// ── 常量 ───────────────────────────────────────────────────────────────

const PROVIDER_META: Record<string, { name: string; color: string }> = {
  openai:    { name: 'OpenAI',    color: '#10B981' },
  anthropic: { name: 'Anthropic', color: '#D97706' },
  google:    { name: 'Google',    color: '#3B82F6' },
  deepseek:  { name: 'DeepSeek',  color: '#8B5CF6' },
};

function getScoreColor(score: number): string {
  if (score >= 9) return '#10B981';
  if (score >= 7) return '#4F46E5';
  if (score >= 5) return '#D97706';
  return '#DC2626';
}

function getScoreLabel(score: number): string {
  if (score >= 9) return '卓越';
  if (score >= 7) return '良好';
  if (score >= 5) return '一般';
  return '较差';
}

function formatContent(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  const slice = text.slice(0, maxLen);
  const lastNewline = slice.lastIndexOf('\n');
  const lastSpace = slice.lastIndexOf(' ');
  const breakAt = lastNewline > maxLen * 0.5 ? lastNewline : Math.max(lastSpace, maxLen * 0.7);
  return text.slice(0, breakAt);
}

function estimateReadTime(content: string): number {
  return Math.max(1, Math.ceil(content.length / 500));
}

const SUGGESTIONS = [
  '用 Rust 实现一个简单的 HTTP 服务器',
  '解释量子计算的基本原理',
  '比较 React 和 Vue 的优缺点',
  '写一篇关于人工智能未来的短文',
];

// ── 主组件 ─────────────────────────────────────────────────────────────

export default function SearchPage() {
  const { t } = useI18n();
  const { isSearching, currentResult, setSearching, setSearchResult, clearResult, addToHistory } = useChatStore();
  const [input, setInput] = useState('');
  const [error, setError] = useState('');
  const [expandedCards, setExpandedCards] = useState<Set<number>>(new Set());
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!currentResult) inputRef.current?.focus();
  }, [currentResult]);

  const handleSearch = async (query?: string) => {
    const q = (query || input).trim();
    if (!q || isSearching) return;
    setError('');
    setSearching(true);
    setExpandedCards(new Set());
    try {
      const resp = await fetchBatchChat([{ role: 'user', content: q }]);
      setSearchResult({
        id: resp.id,
        query: resp.query,
        results: resp.results,
        judgeModel: resp.judge_model,
        totalLatency: resp.total_latency_ms,
        timestamp: Date.now(),
      });
      addToHistory({
        id: resp.id,
        query: resp.query,
        results: resp.results,
        judgeModel: resp.judge_model,
        totalLatency: resp.total_latency_ms,
        timestamp: Date.now(),
      });
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e) || t('common.requestFailed'));
    } finally {
      setSearching(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSearch();
    }
  };

  const toggleCard = (index: number) => {
    setExpandedCards((prev) => {
      const next = new Set(prev);
      next.has(index) ? next.delete(index) : next.add(index);
      return next;
    });
  };

  const handleNewSearch = () => {
    clearResult();
    setError('');
    setTimeout(() => inputRef.current?.focus(), 100);
  };

  // ── 结果视图 ─────────────────────────────────────────────────────────

  if (currentResult) {
    const successResults = currentResult.results.filter((r) => r.success);
    const failedResults = currentResult.results.filter((r) => !r.success);
    const featured = successResults[0] ?? null;
    const rest = successResults.slice(1);
    const hasSideList = rest.length > 0 || failedResults.length > 0;
    const useSplitLayout = featured && hasSideList;

    return (
      <div className="search-page results-mode">
        <div className="search-topbar">
          <div className="search-topbar-inner">
            <span className="search-brand-small" onClick={handleNewSearch}>Nexus AI</span>
            <div className="search-topbar-input">
              <textarea
                value={input || currentResult.query}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                rows={1}
              />
              <button onClick={() => handleSearch()} disabled={isSearching} className="search-topbar-btn">
                {isSearching ? (
                  <span className="search-topbar-spinner" />
                ) : (
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.35-4.35"/></svg>
                )}
              </button>
            </div>
            <span className="search-meta">{currentResult.results.length} 个模型 · {currentResult.totalLatency}ms</span>
          </div>
        </div>

        <div className="search-results">
          {error && <div className="search-error">{error}</div>}

          {isSearching && (
            <div className="search-loading">
              <div className="search-loading-ring">
                <svg className="loading-ring-svg" viewBox="0 0 48 48">
                  <circle cx="24" cy="24" r="20" fill="none" stroke="#E7E5E4" strokeWidth="3" />
                  <circle cx="24" cy="24" r="20" fill="none" stroke="#4F46E5" strokeWidth="3" strokeLinecap="round" strokeDasharray="80 126" />
                </svg>
              </div>
              <p className="search-loading-text">正在对比多个 AI 模型的回答</p>
              <div className="search-loading-models">
                {Object.values(PROVIDER_META).map((p, i) => (
                  <span key={p.name} className="loading-model-tag" style={{ animationDelay: `${i * 0.25}s` }}>
                    <span className="loading-model-dot" style={{ background: p.color }} />
                    {p.name}
                  </span>
                ))}
              </div>
            </div>
          )}

          {!isSearching && successResults.length > 0 && (
            <div className={`results-layout ${useSplitLayout ? 'split' : 'single'}`}>
              {featured && (
                <div className="results-featured">
                  <FeaturedCard
                    result={featured}
                    index={0}
                    expanded={expandedCards.has(0)}
                    onToggle={() => toggleCard(0)}
                  />
                </div>
              )}
              {hasSideList && (
                <div className="results-side">
                  {rest.map((r, i) => (
                    <SideCard
                      key={r.model}
                      result={r}
                      rank={i + 2}
                      index={i + 1}
                      expanded={expandedCards.has(i + 1)}
                      onToggle={() => toggleCard(i + 1)}
                    />
                  ))}
                  {failedResults.map((r) => (
                    <div key={r.model} className="side-card error">
                      <div className="side-card-header">
                        <span className="side-rank">✗</span>
                        <ScoreRing score={0} size={30} />
                        <span className="side-model" style={{ color: '#A8A29E' }}>{r.model}</span>
                      </div>
                      <div className="side-error">{r.error || '请求失败'}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {!isSearching && successResults.length === 0 && !error && (
            <div className="search-empty">没有获得有效回答，请尝试更换问题描述</div>
          )}

          {!isSearching && currentResult.results.length > 0 && (
            <div className="search-footer">
              <span>由 {currentResult.judgeModel} 评分排序</span>
              <span className="footer-dot">·</span>
              <span>总耗时 {(currentResult.totalLatency / 1000).toFixed(1)}s</span>
              <span className="footer-dot">·</span>
              <span onClick={handleNewSearch} className="search-new">发起新搜索</span>
            </div>
          )}
        </div>
      </div>
    );
  }

  // ── 首页视图 ─────────────────────────────────────────────────────────

  return (
    <div className="search-page home-mode">
      <div className="search-home">
        <div className="search-home-logo">
          <div className="home-logo-icon">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
            </svg>
          </div>
        </div>
        <h1 className="search-title">Nexus AI</h1>
        <p className="search-subtitle">同时查询多个 AI，对比最优答案</p>

        <div className="search-box">
          <textarea
            ref={inputRef}
            className="search-input"
            placeholder="输入你的问题..."
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            rows={1}
          />
          <button
            className="search-btn"
            onClick={() => handleSearch()}
            disabled={isSearching || !input.trim()}
          >
            {isSearching ? <span className="search-topbar-spinner" /> : (
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.35-4.35"/></svg>
            )}
          </button>
        </div>

        {error && <div className="search-error">{error}</div>}

        {isSearching && (
          <div className="search-loading search-loading-home">
            <div className="search-loading-ring">
              <svg className="loading-ring-svg" viewBox="0 0 48 48">
                <circle cx="24" cy="24" r="20" fill="none" stroke="#E7E5E4" strokeWidth="3" />
                <circle cx="24" cy="24" r="20" fill="none" stroke="#4F46E5" strokeWidth="3" strokeLinecap="round" strokeDasharray="80 126" />
              </svg>
            </div>
            <p className="search-loading-text">正在对比多个 AI 模型的回答</p>
            <div className="search-loading-models">
              {Object.values(PROVIDER_META).map((p, i) => (
                <span key={p.name} className="loading-model-tag" style={{ animationDelay: `${i * 0.25}s` }}>
                  <span className="loading-model-dot" style={{ background: p.color }} />
                  {p.name}
                </span>
              ))}
            </div>
          </div>
        )}

        {!isSearching && (
          <div className="search-suggestions">
            {SUGGESTIONS.map((s) => (
              <button key={s} className="search-suggestion" onClick={() => { setInput(s); handleSearch(s); }}>
                <span className="suggestion-icon">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M5 12h14M12 5l7 7-7 7"/></svg>
                </span>
                <span className="suggestion-text">{s}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// ── 子组件 ─────────────────────────────────────────────────────────────

function ScoreRing({ score, size = 40 }: { score: number; size?: number }) {
  const color = getScoreColor(score);
  const radius = (size - 6) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (score / 10) * circumference;

  return (
    <div className="score-ring" style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke="#F0EFED" strokeWidth="3" />
        <circle
          cx={size / 2} cy={size / 2} r={radius}
          fill="none" stroke={color} strokeWidth="3"
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          className="score-ring-progress"
        />
      </svg>
      <span className="score-ring-value" style={{ color, fontSize: size < 36 ? '11px' : '13px' }}>
        {score.toFixed(1)}
      </span>
    </div>
  );
}

function FeaturedCard({ result, index, expanded, onToggle }: {
  result: ModelResult;
  index: number;
  expanded: boolean;
  onToggle: () => void;
}) {
  const meta = PROVIDER_META[result.provider] || { name: result.provider, color: '#78716C' };
  const scoreLabel = getScoreLabel(result.score);
  const preview = formatContent(result.content, 600);
  const isLong = result.content.length > 600;
  const readTime = estimateReadTime(result.content);

  return (
    <div className="featured-card">
      <div className="featured-badge-wrap">
        <div className="featured-badge">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
          最佳回答
        </div>
      </div>

      <div className="featured-header">
        <div className="featured-header-left">
          <div className="featured-model-info">
            <span className="provider-dot" style={{ background: meta.color }} />
            <span className="featured-model">{result.model}</span>
            <span className="featured-provider">{meta.name}</span>
          </div>
          <div className="featured-meta-row">
            <span className="featured-latency">{result.latency_ms}ms</span>
            <span className="meta-sep">·</span>
            <span className="featured-read-time">{readTime} min read</span>
          </div>
        </div>
        <div className="featured-score-wrap">
          <ScoreRing score={result.score} size={48} />
          <span className="score-label" style={{ color: getScoreColor(result.score) }}>{scoreLabel}</span>
        </div>
      </div>

      <div className={`featured-content ${expanded ? 'expanded' : ''}`}>
        {expanded ? result.content : preview}
        {!expanded && isLong && <span className="content-ellipsis">...</span>}
      </div>

      <div className="featured-actions">
        {isLong && (
          <button className="expand-btn" onClick={onToggle}>
            {expanded ? '收起回答' : '展开完整回答'}
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ transform: expanded ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }}>
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </button>
        )}
        {result.reason && (
          <button className="reason-toggle" onClick={(e) => {
            const el = (e.target as HTMLElement).closest('.featured-card')?.querySelector('.featured-reason');
            el?.classList.toggle('visible');
          }}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
            评分说明
          </button>
        )}
      </div>

      {result.reason && (
        <div className="featured-reason">
          <span className="reason-label">评分依据：</span>
          {result.reason}
        </div>
      )}
    </div>
  );
}

function SideCard({ result, rank, index, expanded, onToggle }: {
  result: ModelResult;
  rank: number;
  index: number;
  expanded: boolean;
  onToggle: () => void;
}) {
  const meta = PROVIDER_META[result.provider] || { name: result.provider, color: '#78716C' };
  const preview = formatContent(result.content, 200);
  const isLong = result.content.length > 200;

  return (
    <div className="side-card">
      <div className="side-card-header">
        <span className="side-rank">#{rank}</span>
        <ScoreRing score={result.score} size={32} />
        <div className="side-card-info">
          <span className="side-model">{result.model}</span>
          <span className="side-provider">
            <span className="provider-dot" style={{ background: meta.color }} />
            {meta.name}
          </span>
        </div>
        <span className="side-latency">{result.latency_ms}ms</span>
      </div>
      <div className={`side-content ${expanded ? 'expanded' : ''}`}>
        {expanded ? result.content : preview}
        {!expanded && isLong && '...'}
      </div>
      <div className="side-actions">
        {isLong && (
          <button className="side-expand" onClick={onToggle}>
            {expanded ? '收起' : '展开'}
          </button>
        )}
        {result.reason && (
          <span className="side-reason-hint" title={result.reason}>
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
          </span>
        )}
      </div>
      {result.reason && expanded && (
        <div className="side-reason">{result.reason}</div>
      )}
    </div>
  );
}
