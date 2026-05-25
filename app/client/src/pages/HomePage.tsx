import { useState, useEffect, useRef, useCallback } from 'react';
import { useI18n } from '../i18n';
import './HomePage.css';

function maskName(name: string): string {
  if (!name) return '***';
  return name.charAt(0) + '***';
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
  return String(n);
}

type ChartPeriod = 'day' | 'week' | 'month';

interface TimeSeriesModel {
  rank: number;
  name: string;
  provider: string;
  color: string;
  data: number[];
}

const timeSeriesLabels: Record<ChartPeriod, string[]> = {
  day:   ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
  week:  ['W16', 'W17', 'W18', 'W19', 'W20', 'W21', 'W22', 'W23'],
  month: ['Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec', 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun'],
};

const timeSeriesModels: TimeSeriesModel[] = [
  { rank: 1,  name: 'GPT-4o',             provider: 'OpenAI',    color: '#10B981', data: [] },
  { rank: 2,  name: 'Claude 3.5 Sonnet',   provider: 'Anthropic', color: '#F59E0B', data: [] },
  { rank: 3,  name: 'Gemini 1.5 Pro',      provider: 'Google',    color: '#3B82F6', data: [] },
  { rank: 4,  name: 'DeepSeek V3',         provider: 'DeepSeek',  color: '#8B5CF6', data: [] },
  { rank: 5,  name: 'GPT-4 Turbo',         provider: 'OpenAI',    color: '#10B981', data: [] },
  { rank: 6,  name: 'Claude 3 Opus',       provider: 'Anthropic', color: '#F59E0B', data: [] },
  { rank: 7,  name: 'Gemini Flash',        provider: 'Google',    color: '#3B82F6', data: [] },
  { rank: 8,  name: 'Llama 3 70B',         provider: 'Meta',      color: '#EC4899', data: [] },
  { rank: 9,  name: 'Mistral Large',       provider: 'Mistral',   color: '#14B8A6', data: [] },
  { rank: 10, name: 'Command R+',          provider: 'Cohere',    color: '#6366F1', data: [] },
];

function seedTimeSeries(base: number, len: number, variance: number, trend: number): number[] {
  const arr: number[] = [];
  let v = base;
  for (let i = 0; i < len; i++) {
    v = Math.max(v * 0.7, v + (Math.random() - 0.5) * variance + trend);
    arr.push(Math.round(v));
  }
  return arr;
}

const chartDataByPeriod: Record<ChartPeriod, { labels: string[]; models: TimeSeriesModel[] }> = {
  day: {
    labels: timeSeriesLabels.day,
    models: timeSeriesModels.map((m, i) => ({
      ...m,
      data: seedTimeSeries((10 - i) * 60000, 7, 40000, 0),
    })),
  },
  week: {
    labels: timeSeriesLabels.week,
    models: timeSeriesModels.map((m, i) => ({
      ...m,
      data: seedTimeSeries((10 - i) * 420000, 8, 280000, (10 - i) * 12000),
    })),
  },
  month: {
    labels: timeSeriesLabels.month,
    models: timeSeriesModels.map((m, i) => ({
      ...m,
      data: seedTimeSeries((10 - i) * 1800000, 12, 1200000, (10 - i) * 50000),
    })),
  },
};

const newsItems = [
  {
    title: 'OpenAI Unveils GPT-5 with Breakthrough Reasoning Capabilities',
    source: 'TechCrunch',
    date: '2026-05-23',
    snippet: 'The next-generation model demonstrates PhD-level reasoning across mathematics, coding, and scientific research benchmarks.',
    tag: 'Models',
  },
  {
    title: 'Anthropic Secures $8B Funding, Valuation Reaches $80B',
    source: 'The Verge',
    date: '2026-05-21',
    snippet: 'The Claude maker plans to use the new capital to expand its AI safety research and enterprise offerings.',
    tag: 'Industry',
  },
  {
    title: 'EU AI Act Enforcement Begins: What Developers Need to Know',
    source: 'Wired',
    date: '2026-05-20',
    snippet: 'Key compliance requirements take effect, including transparency obligations and risk classification for AI systems.',
    tag: 'Policy',
  },
  {
    title: 'Open-Source Models Surpass Proprietary Counterparts in Latest Benchmarks',
    source: 'Ars Technica',
    date: '2026-05-18',
    snippet: 'Llama 4 and Mistral Large achieve parity with leading commercial models on MMLU, HumanEval and GSM8K tests.',
    tag: 'Research',
  },
];

type LeaderboardPeriod = 'year' | 'month' | 'week';

const leaderboardData: Record<LeaderboardPeriod, Array<{ name: string; tokens: number; rank: number }>> = {
  year: [
    { name: 'Alice Johnson', tokens: 12_500_000, rank: 1 },
    { name: '张伟', tokens: 11_200_000, rank: 2 },
    { name: 'Bob Smith', tokens: 9_800_000, rank: 3 },
    { name: '刘洋', tokens: 8_500_000, rank: 4 },
    { name: 'Charlie Brown', tokens: 7_300_000, rank: 5 },
    { name: '王芳', tokens: 6_900_000, rank: 6 },
    { name: 'Diana Prince', tokens: 5_400_000, rank: 7 },
    { name: '李娜', tokens: 4_800_000, rank: 8 },
    { name: 'Edward Norton', tokens: 3_200_000, rank: 9 },
    { name: '赵敏', tokens: 2_100_000, rank: 10 },
  ],
  month: [
    { name: '张伟', tokens: 1_150_000, rank: 1 },
    { name: 'Alice Johnson', tokens: 980_000, rank: 2 },
    { name: '刘洋', tokens: 870_000, rank: 3 },
    { name: 'Bob Smith', tokens: 750_000, rank: 4 },
    { name: '王芳', tokens: 680_000, rank: 5 },
    { name: 'Diana Prince', tokens: 520_000, rank: 6 },
    { name: '李娜', tokens: 480_000, rank: 7 },
    { name: 'Charlie Brown', tokens: 410_000, rank: 8 },
    { name: '赵敏', tokens: 350_000, rank: 9 },
    { name: 'Edward Norton', tokens: 280_000, rank: 10 },
  ],
  week: [
    { name: '刘洋', tokens: 320_000, rank: 1 },
    { name: '张伟', tokens: 285_000, rank: 2 },
    { name: 'Alice Johnson', tokens: 240_000, rank: 3 },
    { name: '王芳', tokens: 195_000, rank: 4 },
    { name: 'Bob Smith', tokens: 160_000, rank: 5 },
    { name: '李娜', tokens: 140_000, rank: 6 },
    { name: 'Diana Prince', tokens: 115_000, rank: 7 },
    { name: '赵敏', tokens: 95_000, rank: 8 },
    { name: 'Edward Norton', tokens: 72_000, rank: 9 },
    { name: 'Charlie Brown', tokens: 54_000, rank: 10 },
  ],
};

function medalClass(rank: number): string {
  if (rank === 1) return 'medal gold';
  if (rank === 2) return 'medal silver';
  if (rank === 3) return 'medal bronze';
  return 'medal';
}

/* ---- SVG Line Chart ---- */
const CHART_ML = 52;
const CHART_MR = 16;
const CHART_MT = 14;
const CHART_MB = 32;

function svgPath(points: [number, number][], smooth: boolean): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0][0]},${points[0][1]}`;
  let d = `M ${points[0][0]},${points[0][1]}`;
  if (!smooth) {
    for (let i = 1; i < points.length; i++) {
      d += ` L ${points[i][0]},${points[i][1]}`;
    }
    return d;
  }
  for (let i = 0; i < points.length - 1; i++) {
    const [x0, y0] = points[i];
    const [x1, y1] = points[i + 1];
    const dx = (x1 - x0) * 0.35;
    d += ` C ${x0 + dx},${y0} ${x1 - dx},${y1} ${x1},${y1}`;
  }
  return d;
}

function areaPath(linePoints: [number, number][], baselineY: number, w: number): string {
  if (linePoints.length < 2) return '';
  const first = linePoints[0];
  const last = linePoints[linePoints.length - 1];
  return (
    svgPath(linePoints, true) +
    ` L ${last[0]},${baselineY} L ${first[0]},${baselineY} Z`
  );
}

function ModelLineChart({
  data,
  onHoverModel,
  selectedModel,
  onDeselect,
}: {
  data: { labels: string[]; models: TimeSeriesModel[] };
  onHoverModel: (name: string | null) => void;
  selectedModel: string | null;
  onDeselect: () => void;
}) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [hoverX, setHoverX] = useState<number | null>(null);
  const [drawn, setDrawn] = useState(false);
  const [svgW, setSvgW] = useState(700);

  useEffect(() => {
    const timer = setTimeout(() => setDrawn(true), 200);
    return () => clearTimeout(timer);
  }, [data]);

  useEffect(() => {
    const el = svgRef.current?.parentElement;
    if (!el) return;
    const ro = new ResizeObserver(([e]) => {
      if (e) setSvgW(e.contentRect.width);
    });
    ro.observe(el);
    setSvgW(el.clientWidth);
    return () => ro.disconnect();
  }, []);

  const labels = data.labels;
  const models = data.models;
  const allValues = models.flatMap((m) => m.data);
  const minV = Math.min(...allValues);
  const maxV = Math.max(...allValues);
  const pad = Math.max((maxV - minV) * 0.08, 1);
  const yMin = Math.max(0, minV - pad);
  const yMax = maxV + pad;

  const h = 320;
  const pw = svgW - CHART_ML - CHART_MR;
  const ph = h - CHART_MT - CHART_MB;
  const xScale = (i: number) => CHART_ML + (i / (labels.length - 1)) * pw;
  const yScale = (v: number) => CHART_MT + ph - ((v - yMin) / (yMax - yMin)) * ph;
  const yTicks = 5;

  const handleMouse = useCallback(
    (e: React.MouseEvent) => {
      const rect = svgRef.current?.getBoundingClientRect();
      if (!rect) return;
      const mx = e.clientX - rect.left;
      if (mx < CHART_ML || mx > CHART_ML + pw) {
        setHoverX(null);
        onHoverModel(null);
        return;
      }
      setHoverX(mx);
      const nearestIdx = Math.round(((mx - CHART_ML) / pw) * (labels.length - 1));
      const clamped = Math.max(0, Math.min(labels.length - 1, nearestIdx));
      let bestModel: string | null = null;
      let bestVal = -Infinity;
      for (const m of models) {
        if (m.data[clamped] > bestVal) {
          bestVal = m.data[clamped];
          bestModel = m.name;
        }
      }
      onHoverModel(bestModel);
    },
    [labels.length, pw, models, onHoverModel],
  );

  const handleLeave = useCallback(() => {
    setHoverX(null);
    onHoverModel(null);
  }, [onHoverModel]);

  // figure out which x-index the hover snaps to
  const hoverIdx =
    hoverX != null ? Math.round(((hoverX - CHART_ML) / pw) * (labels.length - 1)) : null;
  const clampedHoverIdx =
    hoverIdx != null ? Math.max(0, Math.min(labels.length - 1, hoverIdx)) : null;

  const highlight = useCallback(
    (name: string) => {
      if (!selectedModel) return '';
      return selectedModel === name ? 'hl' : 'dim';
    },
    [selectedModel],
  );

  const top5 = models.slice(0, 5);
  const rest = models.slice(5);

  return (
    <div className="line-chart-wrap">
      <svg
        ref={svgRef}
        viewBox={`0 0 ${svgW} ${h}`}
        className={`line-chart-svg ${drawn ? 'drawn' : ''}`}
        onMouseMove={handleMouse}
        onMouseLeave={handleLeave}
        onClick={onDeselect}
      >
        <defs>
          {models.map((m) => (
            <linearGradient key={m.rank} id={`area-${m.rank}`} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={m.color} stopOpacity="0.28" />
              <stop offset="100%" stopColor={m.color} stopOpacity="0.02" />
            </linearGradient>
          ))}
        </defs>

        {/* grid lines */}
        {Array.from({ length: yTicks }, (_, i) => {
          const v = yMin + ((yMax - yMin) / (yTicks - 1)) * i;
          const y = yScale(v);
          return (
            <g key={`grid-${i}`}>
              <line x1={CHART_ML} x2={CHART_ML + pw} y1={y} y2={y}
                stroke="#E7E5E4" strokeWidth="1" strokeDasharray="4 3" />
              <text x={CHART_ML - 10} y={y + 4} textAnchor="end"
                className="chart-y-label">{formatTokens(v)}</text>
            </g>
          );
        })}

        {/* x labels */}
        {labels.map((l, i) => (
          <text key={`x-${i}`} x={xScale(i)} y={h - 6} textAnchor="middle"
            className="chart-x-label">{l}</text>
        ))}

        {/* area fills (top 5 only) */}
        {top5.map((m) => {
          const pts = m.data.map((v, i) => [xScale(i), yScale(v)] as [number, number]);
          const baseY = yScale(yMin);
          return (
            <path key={`area-${m.rank}`} d={areaPath(pts, baseY, svgW)}
              fill={`url(#area-${m.rank})`} className={`chart-area ${highlight(m.name)}`} />
          );
        })}

        {/* lines */}
        {top5.map((m) => {
          const pts = m.data.map((v, i) => [xScale(i), yScale(v)] as [number, number]);
          return (
            <path key={`line-${m.rank}`} d={svgPath(pts, true)}
              fill="none" stroke={m.color} strokeWidth="2" strokeLinecap="round"
              className={`chart-line ${highlight(m.name)}`} />
          );
        })}
        {rest.map((m) => {
          const pts = m.data.map((v, i) => [xScale(i), yScale(v)] as [number, number]);
          return (
            <path key={`line-${m.rank}`} d={svgPath(pts, true)}
              fill="none" stroke={m.color} strokeWidth="1" strokeOpacity="0.45"
              strokeLinecap="round" className={`chart-line ${highlight(m.name)}`} />
          );
        })}

        {/* hover cursor */}
        {clampedHoverIdx != null && (
          <g>
            <line x1={xScale(clampedHoverIdx)} x2={xScale(clampedHoverIdx)}
              y1={CHART_MT} y2={CHART_MT + ph}
              stroke="#D6D3D1" strokeWidth="1" strokeDasharray="3 2" />
            {models.map((m) => {
              const v = m.data[clampedHoverIdx];
              const cx = xScale(clampedHoverIdx);
              const cy = yScale(v);
              return (
                <circle key={`dot-${m.rank}`} cx={cx} cy={cy} r="3.5"
                  fill="#FFFFFF" stroke={m.color} strokeWidth="2" />
              );
            })}
          </g>
        )}
      </svg>

      {/* tooltip */}
      {clampedHoverIdx != null && (
        <div className="chart-tooltip" style={{
          left: xScale(clampedHoverIdx),
          transform: clampedHoverIdx < labels.length / 2
            ? 'translateX(0)'
            : 'translateX(-100%)',
        }}>
          <div className="chart-tooltip-date">{labels[clampedHoverIdx]}</div>
          {models.slice(0, 5).map((m) => (
            <div key={m.rank} className="chart-tooltip-row">
              <span className="chart-tooltip-dot" style={{ background: m.color }} />
              <span className="chart-tooltip-name">{m.name}</span>
              <span className="chart-tooltip-val">{formatTokens(m.data[clampedHoverIdx])}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/* ---- Legend ---- */
function ChartLegend({
  models,
  hovered,
  selected,
  onSelect,
}: {
  models: TimeSeriesModel[];
  hovered: string | null;
  selected: string | null;
  onSelect: (name: string | null) => void;
}) {
  return (
    <div className="chart-legend">
      {models.map((m) => (
        <button
          key={m.rank}
          className={`chart-legend-item ${hovered === m.name ? 'hl' : ''} ${selected === m.name ? 'sel' : ''} ${(hovered || selected) && hovered !== m.name && selected !== m.name ? 'dim' : ''}`}
          onClick={(e) => { e.stopPropagation(); onSelect(selected === m.name ? null : m.name); }}
          onMouseEnter={() => {}}
        >
          <span className="chart-legend-dot" style={{ background: m.color }} />
          <span className="chart-legend-name">{m.name}</span>
        </button>
      ))}
    </div>
  );
}

export default function HomePage() {
  const { t, locale } = useI18n();
  const [period, setPeriod] = useState<LeaderboardPeriod>('month');
  const [chartPeriod, setChartPeriod] = useState<ChartPeriod>('week');
  const [visible, setVisible] = useState(false);
  const [hoveredModel, setHoveredModel] = useState<string | null>(null);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);

  useEffect(() => {
    const timer = setTimeout(() => setVisible(true), 50);
    return () => clearTimeout(timer);
  }, []);

  const currentRankings = leaderboardData[period];
  const chartData = chartDataByPeriod[chartPeriod];

  return (
    <div className={`homepage ${visible ? 'visible' : ''}`} onClick={() => setSelectedModel(null)}>
      {/* Hero */}
      <section className="hp-hero">
        <div className="hp-hero-text">
          <p className="hp-hero-eyebrow">{t('home.eyebrow')}</p>
          <h1 className="hp-hero-title">{t('home.title')}</h1>
          <p className="hp-hero-subtitle">{t('home.subtitle')}</p>
        </div>
        <div className="hp-hero-stats">
          <div className="hp-stat" style={{ animationDelay: '0.1s' }}>
            <span className="hp-stat-value">12.4M</span>
            <span className="hp-stat-label">{t('home.totalTokens')}</span>
          </div>
          <div className="hp-stat" style={{ animationDelay: '0.2s' }}>
            <span className="hp-stat-value">847</span>
            <span className="hp-stat-label">{t('home.activeUsers')}</span>
          </div>
          <div className="hp-stat" style={{ animationDelay: '0.3s' }}>
            <span className="hp-stat-value">32</span>
            <span className="hp-stat-label">{t('home.availableModels')}</span>
          </div>
        </div>
      </section>

      {/* Main Grid */}
      <section className="hp-main-grid">
        {/* Model Token Rankings */}
        <div className="hp-card hp-chart-card hp-grid-chart" style={{ animationDelay: '0.15s' }}>
          <div className="hp-card-header">
            <h2 className="hp-card-title">{t('home.modelRankings')}</h2>
            <span className="hp-card-badge">{t('home.realtime')}</span>
          </div>
          <div className="hp-chart-tabs">
            {(['day', 'week', 'month'] as ChartPeriod[]).map((p) => (
              <button
                key={p}
                className={`hp-chart-tab ${chartPeriod === p ? 'active' : ''}`}
                onClick={(e) => { e.stopPropagation(); setChartPeriod(p); }}
              >
                {t(`home.${p}`)}
              </button>
            ))}
          </div>
          <ModelLineChart data={chartData} onHoverModel={setHoveredModel} selectedModel={selectedModel} onDeselect={() => setSelectedModel(null)} />
          <ChartLegend models={chartData.models} hovered={hoveredModel} selected={selectedModel} onSelect={setSelectedModel} />
        </div>

        {/* User Leaderboard */}
        <div className="hp-card hp-leaderboard-card hp-grid-leader" style={{ animationDelay: '0.2s' }}>
          <div className="hp-card-header">
            <h2 className="hp-card-title">{t('home.userLeaderboard')}</h2>
          </div>
          <div className="hp-period-tabs">
            {(['year', 'month', 'week'] as LeaderboardPeriod[]).map((p) => (
              <button
                key={p}
                className={`hp-period-tab ${period === p ? 'active' : ''}`}
                onClick={(e) => { e.stopPropagation(); setPeriod(p); }}
              >
                {t(`home.${p}`)}
              </button>
            ))}
          </div>
          <div className="hp-leaderboard-list">
            {currentRankings.map((user) => (
              <div className={`hp-leaderboard-row ${medalClass(user.rank)}`} key={user.rank}>
                <span className="hp-lb-rank">{user.rank}</span>
                <span className="hp-lb-name">{maskName(user.name)}</span>
                <span className="hp-lb-tokens">{formatTokens(user.tokens)}</span>
              </div>
            ))}
          </div>
          <div className="hp-leaderboard-footer">
            <span className="hp-lb-note">{t('home.privacyNote')}</span>
          </div>
        </div>

        {/* AI News */}
        <div className="hp-news-section hp-grid-news" style={{ animationDelay: '0.3s' }}>
          <div className="hp-news-header">
            <h2 className="hp-card-title">{t('home.aiNews')}</h2>
            <a href="#" className="hp-news-more">{t('home.viewAll')} &rarr;</a>
          </div>
          <div className="hp-news-grid">
            {newsItems.map((news, i) => (
              <article className="hp-news-card" key={i} style={{ animationDelay: `${0.35 + i * 0.08}s` }}>
                <span className="hp-news-tag">{news.tag}</span>
                <h3 className="hp-news-title">{news.title}</h3>
                <p className="hp-news-snippet">{news.snippet}</p>
                <div className="hp-news-meta">
                  <span>{news.source}</span>
                  <span>&middot;</span>
                  <span>{new Date(news.date).toLocaleDateString(locale === 'zh' ? 'zh-CN' : 'en-US', { month: 'short', day: 'numeric' })}</span>
                </div>
              </article>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
