/**
 * @file ModelsPage - AI 模型浏览和选择页面
 * 展示所有可用模型，支持按提供商筛选和搜索
 * 选择模型后自动跳转到聊天页面
 */
import { useState, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Search, Layers, ChevronRight, Check, Sparkles } from 'lucide-react';
import { useModelState } from '../stores/modelStore';
import { useChatStore } from '../stores/chatStore';
import { useI18n } from '../i18n';
import './ModelsPage.css';

// 提供商元信息
const PROVIDER_META: Record<string, { color: string; label: string }> = {
  openai: { color: '#10B981', label: 'OpenAI' },
  anthropic: { color: '#D97706', label: 'Anthropic' },
  google: { color: '#3B82F6', label: 'Google' },
  deepseek: { color: '#8B5CF6', label: 'DeepSeek' },
};

/**
 * ModelsPage - 模型浏览主组件
 * @description 展示可用模型，支持搜索和提供商筛选
 */
export default function ModelsPage() {
  const navigate = useNavigate();
  const { models, loaded, loadModels } = useModelState();
  const { selectedModel, setSelectedModel } = useChatStore();
  const { t } = useI18n();
  const [search, setSearch] = useState('');
  const [activeProvider, setActiveProvider] = useState<string | null>(null);

  if (!loaded) loadModels();

  // 从模型列表中提取所有提供商
  const providers = useMemo(() => {
    const set = new Set(models.map((m) => m.provider));
    return Array.from(set);
  }, [models]);

  // 根据搜索关键词和选中的提供商过滤模型
  const filtered = useMemo(() => {
    return models.filter((m) => {
      const matchesSearch = !search ||
        m.name.toLowerCase().includes(search.toLowerCase()) ||
        m.id.toLowerCase().includes(search.toLowerCase());
      const matchesProvider = !activeProvider || m.provider === activeProvider;
      return matchesSearch && matchesProvider;
    });
  }, [models, search, activeProvider]);

  // 选择模型：更新全局选中状态并跳转到聊天页面
  const handleSelect = (modelId: string) => {
    setSelectedModel(modelId);
    navigate('/chat');
  };

  return (
    <div className="models-page">
      {/* 页面头部 */}
      <header className="models-header">
        <div className="models-header-text">
          <h1 className="models-title">{t('models.title')}</h1>
          <p className="models-subtitle">{t('models.subtitle', { count: models.length })}</p>
        </div>
      </header>

      {/* 工具栏：搜索框 + 提供商筛选 */}
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
        {/* 提供商筛选标签 */}
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

      {/* 模型卡片网格 */}
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
