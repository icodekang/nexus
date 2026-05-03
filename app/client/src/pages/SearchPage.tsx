/**
 * @file SearchPage - 多模型搜索对比页面
 * 流程：选择模型回答 → 先随机展示 → 评委打分 → 按分数动态重排
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { useChatStore } from '../stores/chatStore';
import { fetchBatchChat, fetchBatchJudge, type ModelResult } from '../api/client';
import { useI18n } from '../i18n';
import './SearchPage.css';

const PROVIDER_META: Record<string, { name: string; color: string }> = {
  openai:    { name: 'OpenAI',    color: '#10B981' },
  anthropic: { name: 'Anthropic', color: '#D97706' },
  google:    { name: 'Google',    color: '#3B82F6' },
  deepseek:  { name: 'DeepSeek',  color: '#8B5CF6' },
};

const CATEGORY_LABELS: Record<string, string> = {
  code:     '编程',
  creative: '创意写作',
  analysis: '分析推理',
  general:  '通用',
  manual:   '手动选择',
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

function clampContent(text: string, maxLen: number): string {
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

// ── 子组件：评分环 ────────────────────────────────────────────────────────

function ScoreRing({ score, size = 28 }: { score: number; size?: number }) {
  const color = getScoreColor(score);
  const radius = (size - 6) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (score / 10) * circumference;

  return (
    <div className="score-ring" style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        <circle cx={size / 2} cy={size / 2} r={radius} fill="none" stroke="#F0EFED" strokeWidth="2.5" />
        <circle
          cx={size / 2} cy={size / 2} r={radius}
          fill="none" stroke={color} strokeWidth="2.5"
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          className="score-ring-progress"
        />
      </svg>
      <span className="score-ring-value" style={{ color, fontSize: 11 }}>
        {score.toFixed(1)}
      </span>
    </div>
  );
}

// ── 子组件：模型卡片 ─────────────────────────────────────────────────────

function ModelCard({
  result,
  scoring,
  expanded,
  onToggle,
}: {
  result: ModelResult;
  scoring: boolean;
  expanded: boolean;
  onToggle: () => void;
}) {
  const meta = PROVIDER_META[result.provider] || { name: result.provider, color: '#78716C' };
  const hasScore = result.score > 0;
  const preview = clampContent(result.content, 400);
  const isLong = result.content.length > 400;
  const readTime = estimateReadTime(result.content);

  return (
    <div className={`model-card ${!result.success ? 'model-card-error' : ''}`}>
      <div className="model-card-header">
        <div className="model-card-header-left">
          <span className="provider-dot" style={{ background: meta.color }} />
          <span className="model-card-name">{result.model}</span>
          <span className="model-card-provider">{meta.name}</span>
        </div>
        <div className="model-card-header-right">
          {scoring && !hasScore && (
            <span className="model-card-scoring-pulse" title="评委正在评分…" />
          )}
          {hasScore && (
            <div className="model-card-score">
              <ScoreRing score={result.score} size={30} />
              <span className="model-card-score-label" style={{ color: getScoreColor(result.score) }}>
                {getScoreLabel(result.score)}
              </span>
            </div>
          )}
          {!result.success && <span className="model-card-fail-icon">!</span>}
        </div>
      </div>

      <div className="model-card-meta">
        <span className="model-card-latency">{result.latency_ms}ms</span>
        {result.success && (
          <>
            <span className="meta-sep">·</span>
            <span className="model-card-read-time">{readTime} min read</span>
            {result.usage.total_tokens > 0 && (
              <>
                <span className="meta-sep">·</span>
                <span className="model-card-tokens">{result.usage.total_tokens} tokens</span>
              </>
            )}
          </>
        )}
      </div>

      <div className={`model-card-content ${expanded ? 'expanded' : ''}`}>
        {result.success ? (
          <>
            {expanded ? result.content : preview}
            {!expanded && isLong && <span className="content-ellipsis">...</span>}
          </>
        ) : (
          <span className="model-card-error-msg">{result.error || '请求失败'}</span>
        )}
      </div>

      {result.success && (
        <div className="model-card-actions">
          {isLong && (
            <button className="model-card-toggle" onClick={onToggle}>
              {expanded ? '收起回答' : '展开完整回答'}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"
                   strokeLinecap="round" strokeLinejoin="round"
                   style={{ transform: expanded ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }}>
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
          )}
          {result.reason && (
            <button
              className="model-card-reason-btn"
              onClick={(e) => {
                const el = (e.target as HTMLElement).closest('.model-card')?.querySelector('.model-card-reason');
                el?.classList.toggle('visible');
              }}
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"
                   strokeLinecap="round"><circle cx="12" cy="12" r="10" /><path d="M12 16v-4M12 8h.01" /></svg>
              评分说明
            </button>
          )}
        </div>
      )}

      {result.reason && (
        <div className="model-card-reason">
          <span className="reason-label">评分依据：</span>
          {result.reason}
        </div>
      )}
    </div>
  );
}

// ── 主组件 ─────────────────────────────────────────────────────────────────

export default function SearchPage() {
  const { t } = useI18n();
  const {
    isSearching, currentResult, selectedModel, setSelectedModel,
    setSearching, setSearchResult, clearResult, addToHistory,
  } = useChatStore();
  const [input, setInput] = useState('');
  const [error, setError] = useState('');
  const [expandedCards, setExpandedCards] = useState<Set<number>>(new Set());
  const [scoring, setScoring] = useState(false);
  const [judgeModel, setJudgeModel] = useState('');
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!currentResult) inputRef.current?.focus();
  }, [currentResult]);

  const handleSearch = useCallback(async (query?: string) => {
    const q = (query || input).trim();
    if (!q || isSearching) return;
    setError('');
    setSearching(true);
    setScoring(false);
    setJudgeModel('');
    setExpandedCards(new Set());

    const preSelected = selectedModel;
    setSelectedModel(null);

    try {
      const modelsParam = preSelected ? [preSelected] : undefined;
      const resp = await fetchBatchChat([{ role: 'user', content: q }], modelsParam);

      const resultData = {
        id: resp.id,
        query: resp.query,
        results: resp.results,
        judgeModel: resp.judge_model,
        totalLatency: resp.total_latency_ms,
        timestamp: Date.now(),
        selectionCategory: resp.selection_category,
        selectedModels: resp.selected_models,
        hasScoring: resp.has_scoring,
      };
      setSearchResult(resultData);
      addToHistory(resultData);
      setSearching(false);

      // If scoring is available, call judge endpoint asynchronously
      if (resp.has_scoring && resp.results.some((r: ModelResult) => r.success)) {
        setScoring(true);
        try {
          const judgeResp = await fetchBatchJudge({ query: resp.query, results: resp.results });
          setJudgeModel(judgeResp.judge_model);

          const updated = resp.results.map((r: ModelResult) => {
            const scoreInfo = judgeResp.scores.find((s) => s.model === r.model);
            if (scoreInfo) {
              return { ...r, score: scoreInfo.score, reason: scoreInfo.reason };
            }
            return r;
          });
          updated.sort((a: ModelResult, b: ModelResult) => b.score - a.score);
          setSearchResult({
            ...resultData,
            results: updated,
            judgeModel: judgeResp.judge_model,
          });
        } catch {
          // Scoring failed silently — show results without scores
        } finally {
          setScoring(false);
        }
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e) || t('common.requestFailed'));
      setSearching(false);
    }
  }, [input, isSearching, selectedModel, setSelectedModel, setSearching, setSearchResult, addToHistory, t]);

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
    setScoring(false);
    setJudgeModel('');
    setTimeout(() => inputRef.current?.focus(), 100);
  };

  // ── 结果视图 ────────────────────────────────────────────────────────────

  if (currentResult) {
    const successResults = currentResult.results.filter((r) => r.success);
    const failedResults = currentResult.results.filter((r) => !r.success);
    const categoryLabel = CATEGORY_LABELS[currentResult.selectionCategory] || currentResult.selectionCategory;

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
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"
                       strokeLinecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.35-4.35" /></svg>
                )}
              </button>
            </div>
            <span className="search-meta">
              {currentResult.results.length} 个模型 · {currentResult.totalLatency}ms
            </span>
          </div>
        </div>

        <div className="search-results">
          {error && <div className="search-error">{error}</div>}

          {isSearching && (
            <div className="search-loading">
              <div className="search-loading-ring">
                <svg className="loading-ring-svg" viewBox="0 0 48 48">
                  <circle cx="24" cy="24" r="20" fill="none" stroke="#E7E5E4" strokeWidth="3" />
                  <circle cx="24" cy="24" r="20" fill="none" stroke="#4F46E5" strokeWidth="3"
                          strokeLinecap="round" strokeDasharray="80 126" />
                </svg>
              </div>
            </div>
          )}

          {/* Selection info banner */}
          {!isSearching && currentResult.selectedModels.length > 0 && (
            <div className="selection-banner">
              <div className="selection-banner-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"
                     strokeLinecap="round" strokeLinejoin="round">
                  <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
                </svg>
              </div>
              <span>
                针对 <strong>{categoryLabel}</strong> 类问题，选择了{' '}
                <strong>{currentResult.selectedModels.length}</strong> 个最擅长的模型回答
              </span>
            </div>
          )}

          {/* Scoring status banner */}
          {!isSearching && scoring && (
            <div className="selection-banner scoring-banner">
              <div className="scoring-pulse-dot" />
              <span>评委 <strong>{currentResult.judgeModel || 'GPT-4o'}</strong> 正在对回答进行评分排序…</span>
            </div>
          )}

          {/* Results grid */}
          {!isSearching && successResults.length > 0 && (
            <div className="results-grid">
              {successResults.map((r, i) => (
                <ModelCard
                  key={r.model}
                  result={r}
                  scoring={scoring}
                  expanded={expandedCards.has(i)}
                  onToggle={() => toggleCard(i)}
                />
              ))}
            </div>
          )}

          {/* Failed results */}
          {!isSearching && failedResults.length > 0 && (
            <div className="results-grid">
              {failedResults.map((r) => (
                <ModelCard
                  key={r.model}
                  result={r}
                  scoring={false}
                  expanded={false}
                  onToggle={() => {}}
                />
              ))}
            </div>
          )}

          {!isSearching && successResults.length === 0 && !error && (
            <div className="search-empty">没有获得有效回答，请尝试更换问题描述</div>
          )}

          {!isSearching && currentResult.results.length > 0 && (
            <div className="search-footer">
              {currentResult.hasScoring && (
                <>
                  <span>由 {judgeModel || currentResult.judgeModel || 'GPT-4o'} 评分排序</span>
                  <span className="footer-dot">·</span>
                </>
              )}
              <span>总耗时 {(currentResult.totalLatency / 1000).toFixed(1)}s</span>
              <span className="footer-dot">·</span>
              <span onClick={handleNewSearch} className="search-new">发起新搜索</span>
            </div>
          )}
        </div>
      </div>
    );
  }

  // ── 首页视图 ────────────────────────────────────────────────────────────

  return (
    <div className="search-page home-mode">
      <div className="search-home">
        <div className="search-home-logo">
          <div className="home-logo-icon">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"
                 strokeLinecap="round" strokeLinejoin="round">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
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
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"
                   strokeLinecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.35-4.35" /></svg>
            )}
          </button>
        </div>

        {error && <div className="search-error">{error}</div>}

        {isSearching && (
          <div className="search-loading search-loading-home">
            <div className="search-loading-ring">
              <svg className="loading-ring-svg" viewBox="0 0 48 48">
                <circle cx="24" cy="24" r="20" fill="none" stroke="#E7E5E4" strokeWidth="3" />
                <circle cx="24" cy="24" r="20" fill="none" stroke="#4F46E5" strokeWidth="3"
                        strokeLinecap="round" strokeDasharray="80 126" />
              </svg>
            </div>
          </div>
        )}

        {!isSearching && (
          <div className="search-suggestions">
            {SUGGESTIONS.map((s) => (
              <button key={s} className="search-suggestion" onClick={() => { setInput(s); handleSearch(s); }}>
                <span className="suggestion-icon">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"
                       strokeLinecap="round"><path d="M5 12h14M12 5l7 7-7 7" /></svg>
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
