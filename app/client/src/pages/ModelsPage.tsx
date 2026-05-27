import { useState, useMemo, useEffect } from 'react';
import { Search, Filter, X, ArrowUpDown, Layers, ChevronDown, SlidersHorizontal, Server, Type, Image, Volume2, File, Film } from 'lucide-react';
import { useModelState } from '../stores/modelStore';
import { useI18n } from '../i18n';
import type { Model } from '../api/client';
import './ModelsPage.css';
import openaiIcon from '../assets/icons/OpenAI.svg';
import anthropicIcon from '../assets/icons/Anthropic.svg';
import geminiIcon from '../assets/icons/GoogleGemini.svg';
import deepseekIcon from '../assets/icons/DeepSeek.png';

const PROVIDER_META: Record<string, { color: string; label: string; glow: string; logo: React.ReactNode }> = {
  openai: {
    color: '#34D399',
    label: 'OpenAI',
    glow: 'rgba(52,211,153,0.25)',
    logo: <img src={openaiIcon} className="provider-logo-img" alt="OpenAI" />,
  },
  anthropic: {
    color: '#F59E0B',
    label: 'Anthropic',
    glow: 'rgba(245,158,11,0.25)',
    logo: <img src={anthropicIcon} className="provider-logo-img" alt="Anthropic" />,
  },
  google: {
    color: '#60A5FA',
    label: 'Google',
    glow: 'rgba(96,165,250,0.25)',
    logo: <img src={geminiIcon} className="provider-logo-img" alt="Google" />,
  },
  deepseek: {
    color: '#A78BFA',
    label: 'DeepSeek',
    glow: 'rgba(167,139,250,0.25)',
    logo: <img src={deepseekIcon} className="provider-logo-img" alt="DeepSeek" />,
  },
};
const FALLBACK_PROVIDER = { color: '#78716C', label: '', glow: 'rgba(120,113,108,0.2)', logo: null as React.ReactNode };

const MODEL_LOGOS: Record<string, React.ReactNode> = {
  /* ── OpenAI / ChatGPT ── */
  'gpt-4o': (
    <img src={openaiIcon} className="provider-logo-img" alt="GPT-4o" />
  ),
  'gpt-4o-mini': (
    <img src={openaiIcon} className="provider-logo-img" alt="GPT-4o Mini" />
  ),
  'gpt-4-turbo': (
    <img src={openaiIcon} className="provider-logo-img" alt="GPT-4 Turbo" />
  ),
  'gpt-3.5-turbo': (
    <img src={openaiIcon} className="provider-logo-img" alt="GPT-3.5 Turbo" />
  ),

  /* ── Anthropic / Claude ── */
  'claude-3-5-sonnet': (
    <img src={anthropicIcon} className="provider-logo-img" alt="Claude Sonnet" />
  ),
  'claude-3-opus': (
    <img src={anthropicIcon} className="provider-logo-img" alt="Claude Opus" />
  ),
  'claude-3-haiku': (
    <img src={anthropicIcon} className="provider-logo-img" alt="Claude Haiku" />
  ),

  /* ── Google Gemini ── */
  'gemini-1-5-pro': (
    <img src={geminiIcon} className="provider-logo-img" alt="Gemini Pro" />
  ),
  'gemini-1-5-flash': (
    <img src={geminiIcon} className="provider-logo-img" alt="Gemini Flash" />
  ),
  'gemini-1-0-pro': (
    <img src={geminiIcon} className="provider-logo-img" alt="Gemini" />
  ),

  /* ── DeepSeek ── */
  'deepseek-chat': (
    <img src={deepseekIcon} className="provider-logo-img" alt="DeepSeek" />
  ),
  'deepseek-coder': (
    <img src={deepseekIcon} className="provider-logo-img" alt="DeepSeek Coder" />
  ),
};

const INPUT_MODALITY_OPTIONS = [
  { key: 'text', labelKey: 'models.modalities.text', icon: Type },
  { key: 'image', labelKey: 'models.modalities.image', icon: Image },
  { key: 'audio', labelKey: 'models.modalities.audio', icon: Volume2 },
  { key: 'video', labelKey: 'models.modalities.video', icon: Film },
  { key: 'file', labelKey: 'models.modalities.file', icon: File },
];

function getModalities(capabilities: string[]): string[] {
  const mods: string[] = ['text'];
  if (!capabilities || capabilities.length === 0) return mods;
  const capSet = new Set(capabilities.map((c) => c.toLowerCase()));
  if (capSet.has('vision')) mods.push('image');
  if (capSet.has('function_call')) mods.push('file');
  return mods;
}

type SortField = 'name' | 'context_window';
type SortDir = 'asc' | 'desc';

export default function ModelsPage() {
  const { models, loaded, loadModels } = useModelState();
  const { t } = useI18n();

  const [search, setSearch] = useState('');
  const [activeProviders, setActiveProviders] = useState<Set<string>>(new Set());
  const [activeModalities, setActiveModalities] = useState<Set<string>>(new Set());
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [modalitiesOpen, setModalitiesOpen] = useState(false);
  const [providersOpen, setProvidersOpen] = useState(false);
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortDir, setSortDir] = useState<SortDir>('asc');

  useEffect(() => {
    if (!loaded) loadModels();
  }, [loaded, loadModels]);

  const providers = useMemo(() => {
    const set = new Set(models.map((m) => m.provider));
    return Array.from(set);
  }, [models]);

  const filtered = useMemo(() => {
    let result = models.filter((m) => {
      const matchesSearch =
        !search ||
        m.name.toLowerCase().includes(search.toLowerCase()) ||
        m.id.toLowerCase().includes(search.toLowerCase()) ||
        (m.provider_name || '').toLowerCase().includes(search.toLowerCase());
      const matchesProvider = activeProviders.size === 0 || activeProviders.has(m.provider);
      const matchesModality =
        activeModalities.size === 0 ||
        getModalities(m.capabilities).some((mod) => activeModalities.has(mod));
      return matchesSearch && matchesProvider && matchesModality;
    });

    result.sort((a, b) => {
      let cmp = 0;
      if (sortField === 'name') {
        cmp = a.name.localeCompare(b.name);
      } else if (sortField === 'context_window') {
        cmp = a.context_window - b.context_window;
      }
      return sortDir === 'asc' ? cmp : -cmp;
    });

    return result;
  }, [models, search, activeProviders, activeModalities, sortField, sortDir]);

  const toggleProvider = (slug: string) => {
    setActiveProviders((prev) => {
      const next = new Set(prev);
      if (next.has(slug)) next.delete(slug);
      else next.add(slug);
      return next;
    });
  };

  const toggleModality = (key: string) => {
    setActiveModalities((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  const clearFilters = () => {
    setSearch('');
    setActiveProviders(new Set());
    setActiveModalities(new Set());
  };

  const toggleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortField(field);
      setSortDir('asc');
    }
  };

  const hasFilters = search || activeProviders.size > 0 || activeModalities.size > 0;

  const fmtContext = (ctx: number): { value: string; pct: string } => {
    if (ctx >= 1000000) return { value: `${(ctx / 1000000).toFixed(1)}M`, pct: '100' };
    if (ctx >= 200000) return { value: `${(ctx / 1000).toFixed(0)}K`, pct: '90' };
    if (ctx >= 128000) return { value: `${(ctx / 1000).toFixed(0)}K`, pct: '72' };
    if (ctx >= 64000) return { value: `${(ctx / 1000).toFixed(0)}K`, pct: '50' };
    if (ctx >= 32000) return { value: `${(ctx / 1000).toFixed(0)}K`, pct: '35' };
    return { value: `${(ctx / 1000).toFixed(0)}K`, pct: '20' };
  };

  return (
    <div className="models-page">
      {/* Mobile filter toggle */}
      <button className="models-mobile-filter" onClick={() => setSidebarOpen(!sidebarOpen)}>
        <Filter size={14} /> {t('models.filter')}
      </button>

      {sidebarOpen && <div className="models-sidebar-overlay" onClick={() => setSidebarOpen(false)} />}

      {/* ---- SIDEBAR ---- */}
      <aside className={`models-sidebar ${sidebarOpen ? 'open' : ''}`}>
        <div className="models-sidebar-header">
          <button className="models-sidebar-close" onClick={() => setSidebarOpen(false)}>
            <X size={16} />
          </button>
        </div>

        {/* Input Modalities — collapsible dropdown */}
        <div className="models-sidebar-section">
          <button
            className={`models-sidebar-dropdown ${modalitiesOpen ? 'open' : ''}`}
            onClick={() => setModalitiesOpen(!modalitiesOpen)}
          >
            <span className="models-sidebar-dropdown-label">
              <SlidersHorizontal size={14} className="models-sidebar-dropdown-icon" />
              {t('models.inputModalities')}
              {activeModalities.size > 0 && (
                <span className="models-sidebar-dropdown-count">{activeModalities.size}</span>
              )}
            </span>
            <ChevronDown size={14} className="models-sidebar-dropdown-arrow" />
          </button>
          {modalitiesOpen && (
            <div className="models-sidebar-dropdown-panel">
              {INPUT_MODALITY_OPTIONS.map((opt) => {
                const Icon = opt.icon;
                return (
                <label key={opt.key} className="models-sidebar-checkbox">
                  <input
                    type="checkbox"
                    checked={activeModalities.has(opt.key)}
                    onChange={() => toggleModality(opt.key)}
                  />
                  <Icon size={14} className="models-modality-icon" />
                  <span className="models-sidebar-check-label">{t(opt.labelKey)}</span>
                </label>
                );
              })}
            </div>
          )}
        </div>

        {/* Providers — collapsible dropdown */}
        <div className="models-sidebar-section">
          <button
            className={`models-sidebar-dropdown ${providersOpen ? 'open' : ''}`}
            onClick={() => setProvidersOpen(!providersOpen)}
          >
            <span className="models-sidebar-dropdown-label">
              <Server size={14} className="models-sidebar-dropdown-icon" />
              {t('models.providers')}
              {activeProviders.size > 0 && (
                <span className="models-sidebar-dropdown-count">{activeProviders.size}</span>
              )}
            </span>
            <ChevronDown size={14} className="models-sidebar-dropdown-arrow" />
          </button>
          {providersOpen && (
            <div className="models-sidebar-dropdown-panel">
              {providers.map((slug) => {
                const meta = PROVIDER_META[slug] || FALLBACK_PROVIDER;
                return (
                  <label key={slug} className="models-sidebar-checkbox">
                    <input
                      type="checkbox"
                      checked={activeProviders.has(slug)}
                      onChange={() => toggleProvider(slug)}
                    />
                    <span className="models-provider-dot">
                      {meta.logo}
                    </span>
                    <span className="models-sidebar-check-label">{meta.label || slug}</span>
                  </label>
                );
              })}
            </div>
          )}
        </div>
      </aside>

      {/* ---- MAIN ---- */}
      <main className="models-content">
        {/* Header */}
        <div className="models-content-header">
          <div className="models-content-header-left">
            <h1 className="models-content-title">{t('models.title')}</h1>
            <span className="models-count">{t('models.subtitle', { count: models.length })}</span>
          </div>
          <div className="models-content-search">
            <Search size={14} className="models-search-icon" />
            <input
              placeholder={t('models.searchPlaceholder')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            {search && (
              <button className="models-search-clear" onClick={() => setSearch('')}>
                <X size={12} />
              </button>
            )}
          </div>
        </div>

        {/* Active filter chips */}
        {hasFilters && (
          <div className="models-active-filters">
            {search && (
              <span className="models-filter-chip">
                &ldquo;{search}&rdquo;
                <button onClick={() => setSearch('')}><X size={10} /></button>
              </span>
            )}
            {Array.from(activeModalities).map((mod) => (
              <span key={mod} className="models-filter-chip">
                {t(INPUT_MODALITY_OPTIONS.find((o) => o.key === mod)?.labelKey || '')}
                <button onClick={() => toggleModality(mod)}><X size={10} /></button>
              </span>
            ))}
            {Array.from(activeProviders).map((slug) => (
              <span key={slug} className="models-filter-chip">
                {PROVIDER_META[slug]?.label || slug}
                <button onClick={() => toggleProvider(slug)}><X size={10} /></button>
              </span>
            ))}
            <button className="models-filter-clear-all" onClick={clearFilters}>
              {t('models.clearFilters')}
            </button>
          </div>
        )}

        {/* Model rows */}
        <div className="models-list">
          {filtered.map((model, idx) => {
            const meta = PROVIDER_META[model.provider] || FALLBACK_PROVIDER;
            const label = meta.label || model.provider || model.provider_name;
            const mods = getModalities(model.capabilities);
            const ctx = fmtContext(model.context_window);
            const modelIcon = MODEL_LOGOS[model.id] || meta.logo;
            return (
              <div
                key={model.id}
                className="models-row"
                style={{ animationDelay: `${idx * 35}ms` }}
              >
                <div className="models-row-left">
                  <div className="models-row-provider-logo">
                    {modelIcon}
                  </div>
                  <div className="models-row-info">
                    <div className="models-row-name-row">
                      <span className="models-row-name">{model.name}</span>
                    </div>
                    <div className="models-row-meta">
                      <span className="models-row-provider-tag" style={{ color: meta.color }}>
                        {label}
                      </span>
                      <span className="models-row-sep">/</span>
                      <span className="models-row-slug">{model.id}</span>
                    </div>
                    {model.description && (
                      <p className="models-row-desc">{model.description}</p>
                    )}
                    <div className="models-row-modalities">
                      {mods.map((modKey) => (
                        <span key={modKey} className="models-row-mod-tag">
                          {t(INPUT_MODALITY_OPTIONS.find((o) => o.key === modKey)?.labelKey || '')}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
                <div className="models-row-right">
                  <div className="models-row-context" title={t('models.contextWindowRaw', { size: model.context_window })}>
                    <div className="models-row-context-bar">
                      <div
                        className="models-row-context-fill"
                        style={{ width: `${ctx.pct}%` }}
                      />
                    </div>
                    <span className="models-row-context-value">{ctx.value}</span>
                  </div>
                </div>
              </div>
            );
          })}

          {filtered.length === 0 && (
            <div className="models-empty">
              <div className="models-empty-ring" />
              <p>{t('models.noModelsFound')}</p>
              {hasFilters && (
                <button className="models-empty-clear" onClick={clearFilters}>
                  {t('models.clearFilters')}
                </button>
              )}
            </div>
          )}
        </div>
      </main>

      {/* Mobile FAB */}
      <button className="models-mobile-fab" onClick={() => setSidebarOpen(true)}>
        <Filter size={16} />
        <span>{t('models.filter')}</span>
      </button>
    </div>
  );
}
