import { useState, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Search, Layers, ChevronRight, Check, Sparkles } from 'lucide-react';
import { useModelState } from '../stores/modelStore';
import { useChatStore } from '../stores/chatStore';
import { useI18n } from '../i18n';
import './ModelsPage.css';

const PROVIDER_META: Record<string, { color: string; label: string }> = {
  openai: { color: '#10B981', label: 'OpenAI' },
  anthropic: { color: '#D97706', label: 'Anthropic' },
  google: { color: '#3B82F6', label: 'Google' },
  deepseek: { color: '#8B5CF6', label: 'DeepSeek' },
};

export default function ModelsPage() {
  const navigate = useNavigate();
  const { models, loaded, loadModels } = useModelState();
  const { selectedModel, setSelectedModel } = useChatStore();
  const { t } = useI18n();
  const [search, setSearch] = useState('');
  const [activeProvider, setActiveProvider] = useState<string | null>(null);

  if (!loaded) loadModels();

  const providers = useMemo(() => {
    const set = new Set(models.map((m) => m.provider));
    return Array.from(set);
  }, [models]);

  const filtered = useMemo(() => {
    return models.filter((m) => {
      const matchesSearch = !search ||
        m.name.toLowerCase().includes(search.toLowerCase()) ||
        m.id.toLowerCase().includes(search.toLowerCase());
      const matchesProvider = !activeProvider || m.provider === activeProvider;
      return matchesSearch && matchesProvider;
    });
  }, [models, search, activeProvider]);

  const handleSelect = (modelId: string) => {
    setSelectedModel(modelId);
    navigate('/chat');
  };

  return (
    <div className="models-page">
      <header className="models-header">
        <div className="models-header-text">
          <h1 className="models-title">{t('models.title')}</h1>
          <p className="models-subtitle">{t('models.subtitle', { count: models.length })}</p>
        </div>
      </header>

      <div className="models-toolbar">
        <div className="models-search-wrapper">
          <Search size={16} className="models-search-icon" />
          <input
            className="models-search"
            placeholder={t('models.searchPlaceholder')}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="models-filters">
          <button
            className={`models-filter-pill ${!activeProvider ? 'active' : ''}`}
            onClick={() => setActiveProvider(null)}
          >
            {t('common.all')}
          </button>
          {providers.map((p) => (
            <button
              key={p}
              className={`models-filter-pill ${activeProvider === p ? 'active' : ''}`}
              onClick={() => setActiveProvider(activeProvider === p ? null : p)}
              style={activeProvider === p ? { background: PROVIDER_META[p]?.color || '#6366F1', borderColor: PROVIDER_META[p]?.color || '#6366F1', color: '#fff' } : {}}
            >
              <span
                className="models-filter-dot"
                style={{ background: PROVIDER_META[p]?.color || '#A8A29E' }}
              />
              {PROVIDER_META[p]?.label || p}
            </button>
          ))}
        </div>
      </div>

      <div className="models-grid">
        {filtered.map((model) => {
          const isSelected = model.id === selectedModel;
          const meta = PROVIDER_META[model.provider] || { color: '#A8A29E', label: model.provider };
          return (
            <button
              key={model.id}
              className={`models-card ${isSelected ? 'selected' : ''}`}
              onClick={() => handleSelect(model.id)}
            >
              <div className="models-card-header">
                <div className="models-card-provider" style={{ background: meta.color + '18', color: meta.color }}>
                  <span className="models-card-provider-dot" style={{ background: meta.color }} />
                  {meta.label}
                </div>
                {isSelected && (
                  <div className="models-card-selected">
                    <Check size={12} />
                    {t('models.active')}
                  </div>
                )}
              </div>
              <h3 className="models-card-name">{model.name}</h3>
              <p className="models-card-id">{model.id}</p>
              <div className="models-card-meta">
                <span className="models-card-context">
                  <Layers size={12} />
                  {t('models.contextWindow', { size: (model.context_window / 1000).toFixed(0) })}
                </span>
                {model.capabilities?.length > 0 && (
                  <div className="models-card-capabilities">
                    {model.capabilities.map((cap) => (
                      <span key={cap} className="models-card-cap-tag">{cap}</span>
                    ))}
                  </div>
                )}
              </div>
              <ChevronRight size={14} className="models-card-arrow" />
            </button>
          );
        })}
        {filtered.length === 0 && (
          <div className="models-empty">
            <Sparkles size={28} strokeWidth={1.5} />
            <p>{t('models.noModelsFound')}</p>
          </div>
        )}
      </div>
    </div>
  );
}
