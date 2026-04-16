/**
 * @file Models - AI 模型管理页面
 * 展示模型列表，支持添加、编辑、删除模型
 * 模型关联到不同的 AI 服务提供商
 */
import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import { fetchModels, createModel, updateModel, deleteModel, fetchProviders, type AdminModel, type AdminProvider } from '../api/admin';

/**
 * Models - 模型管理主组件
 * @description 获取模型和提供商列表，支持添加/编辑/删除模型
 */
export default function Models() {
  const { t } = useI18n();
  const [models, setModels] = useState<AdminModel[]>([]);
  const [providers, setProviders] = useState<AdminProvider[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [editModel, setEditModel] = useState<AdminModel | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AdminModel | null>(null);

  // 表单状态：模型名称、slug、模型 ID、提供商 ID、上下文窗口、能力列表
  const [formName, setFormName] = useState('');
  const [formSlug, setFormSlug] = useState('');
  const [formModelId, setFormModelId] = useState('');
  const [formProviderId, setFormProviderId] = useState('');
  const [formContext, setFormContext] = useState('');
  const [formCaps, setFormCaps] = useState('');

  // 加载模型和提供商数据
  const loadData = useCallback(() => {
    setLoading(true);
    Promise.all([fetchModels(), fetchProviders()])
      .then(([modelsRes, providersRes]) => {
        setModels(modelsRes.data);
        setProviders(providersRes.data);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const getProviderName = (providerId: string) => {
    const p = providers.find((p) => p.id === providerId);
    return p?.name || providerId;
  };

  // 根据 slug 获取提供商对应的品牌颜色
  const getProviderColor = (providerId: string) => {
    const colors: Record<string, string> = {
      openai: '#10A37F', anthropic: '#D97706', google: '#4285F4', deepseek: '#6366F1',
    };
    const p = providers.find((p) => p.id === providerId);
    return (p && colors[p.slug]) || '#A1A1AA';
  };

  // 格式化上下文窗口大小（K/M 单位）
  const formatContext = (cw: number) => {
    if (cw >= 1_000_000) return `${(cw / 1_000_000).toFixed(0)}M`;
    if (cw >= 1000) return `${(cw / 1000).toFixed(0)}K`;
    return String(cw);
  };

  // 重置表单为空
  const resetForm = () => {
    setFormName('');
    setFormSlug('');
    setFormModelId('');
    setFormProviderId('');
    setFormContext('');
    setFormCaps('');
  };

  const openAddModal = () => {
    resetForm();
    setShowAddModal(true);
  };

  const openEditModal = (m: AdminModel) => {
    setFormName(m.name);
    setFormSlug(m.slug);
    setFormModelId(m.model_id);
    setFormProviderId(m.provider_id);
    setFormContext(String(m.context_window));
    setFormCaps(m.capabilities.join(', '));
    setEditModel(m);
  };

  const handleAddModel = async () => {
    if (!formName.trim() || !formProviderId) return;
    try {
      await createModel({
        provider_id: formProviderId,
        name: formName,
        slug: formSlug || formName.toLowerCase().replace(/\s+/g, '-'),
        model_id: formModelId || formSlug || formName.toLowerCase().replace(/\s+/g, '-'),
        context_window: parseInt(formContext) || 4096,
        capabilities: formCaps.split(',').map((c) => c.trim()).filter(Boolean),
      });
      setShowAddModal(false);
      loadData();
    } catch {
      // Error handling
    }
  };

  const handleEditModel = async () => {
    if (!editModel || !formName.trim()) return;
    try {
      await updateModel(editModel.id, {
        name: formName,
        slug: formSlug,
        model_id: formModelId,
        context_window: parseInt(formContext) || editModel.context_window,
        capabilities: formCaps.split(',').map((c) => c.trim()).filter(Boolean),
      });
      setEditModel(null);
      loadData();
    } catch {
      // Error handling
    }
  };

  const handleDeleteModel = async () => {
    if (!deleteTarget) return;
    try {
      await deleteModel(deleteTarget.id);
      setDeleteTarget(null);
      loadData();
    } catch {
      // Error handling
    }
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('models.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? 'Loading...' : t('models.subtitle', { count: models.length })}
          </p>
        </div>
        <button style={styles.addBtn} onClick={openAddModal}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          {t('models.addModel')}
        </button>
      </header>

      <div style={styles.tableCard}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={{ ...styles.th, paddingLeft: '20px' }}>{t('models.thModel')}</th>
              <th style={styles.th}>{t('models.thProvider')}</th>
              <th style={styles.th}>{t('models.thContext')}</th>
              <th style={styles.th}>{t('models.thCapabilities')}</th>
              <th style={styles.th}>{t('users.thStatus')}</th>
              <th style={{ ...styles.th, paddingRight: '20px', textAlign: 'right' }}></th>
            </tr>
          </thead>
          <tbody>
            {models.map((m) => {
              const color = getProviderColor(m.provider_id);
              return (
                <tr key={m.id} style={styles.tr}>
                  <td style={{ ...styles.td, paddingLeft: '20px' }}>
                    <div>
                      <span style={styles.modelName}>{m.name}</span>
                      <span style={styles.modelSlug}>{m.model_id}</span>
                    </div>
                  </td>
                  <td style={styles.td}>
                    <span style={{
                      ...styles.providerBadge,
                      color,
                      backgroundColor: `${color}12`,
                    }}>
                      {getProviderName(m.provider_id)}
                    </span>
                  </td>
                  <td style={styles.td}>
                    <span style={styles.context}>{formatContext(m.context_window)}</span>
                  </td>
                  <td style={styles.td}>
                    <div style={styles.caps}>
                      {m.capabilities.length > 0 ? m.capabilities.map((c) => (
                        <span key={c} style={styles.capTag}>{c}</span>
                      )) : <span style={styles.noCap}>-</span>}
                    </div>
                  </td>
                  <td style={styles.td}>
                    <span style={{
                      ...styles.status,
                      color: m.is_active ? '#22C55E' : '#EF4444',
                    }}>
                      <span style={{
                        ...styles.statusDot,
                        backgroundColor: m.is_active ? '#22C55E' : '#EF4444',
                      }} />
                      {m.is_active ? t('common.active') : t('common.inactive')}
                    </span>
                  </td>
                  <td style={{ ...styles.td, paddingRight: '20px', textAlign: 'right' }}>
                    <div style={styles.actions}>
                      <button style={styles.actionBtn} onClick={() => openEditModal(m)}>{t('common.edit')}</button>
                      <button
                        style={{ ...styles.actionBtn, color: '#EF4444', borderColor: 'rgba(239, 68, 68, 0.2)' }}
                        onClick={() => setDeleteTarget(m)}
                      >
                        {t('common.delete')}
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
            {!loading && models.length === 0 && (
              <tr>
                <td colSpan={6} style={{ ...styles.td, textAlign: 'center', color: '#A1A1AA', padding: '40px' }}>
                  No models yet
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <Modal open={showAddModal} onClose={() => setShowAddModal(false)} title={t('models.addModel')}>
        <ModelForm
          name={formName} setName={setFormName}
          slug={formSlug} setSlug={setFormSlug}
          modelId={formModelId} setModelId={setFormModelId}
          providerId={formProviderId} setProviderId={setFormProviderId}
          context={formContext} setContext={setFormContext}
          caps={formCaps} setCaps={setFormCaps}
          providers={providers}
          t={t}
          onSubmit={handleAddModel}
          onCancel={() => setShowAddModal(false)}
          submitLabel={t('models.addModel')}
        />
      </Modal>

      <Modal open={!!editModel} onClose={() => setEditModel(null)} title={t('models.editModel')}>
        <ModelForm
          name={formName} setName={setFormName}
          slug={formSlug} setSlug={setFormSlug}
          modelId={formModelId} setModelId={setFormModelId}
          providerId={formProviderId} setProviderId={setFormProviderId}
          context={formContext} setContext={setFormContext}
          caps={formCaps} setCaps={setFormCaps}
          providers={providers}
          t={t}
          onSubmit={handleEditModel}
          onCancel={() => setEditModel(null)}
          submitLabel={t('common.save')}
        />
      </Modal>

      <Modal open={!!deleteTarget} onClose={() => setDeleteTarget(null)} title={t('common.delete')} width={380}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <p style={{ fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif" }}>
            {t('common.deleteConfirm')}
          </p>
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
            <button style={formStyles.cancelBtn} onClick={() => setDeleteTarget(null)}>
              {t('common.cancel')}
            </button>
            <button style={{ ...formStyles.submitBtn, backgroundColor: '#EF4444' }} onClick={handleDeleteModel}>
              {t('common.delete')}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function ModelForm({
  name, setName, slug, setSlug, modelId, setModelId,
  providerId, setProviderId, context, setContext, caps, setCaps,
  providers, t, onSubmit, onCancel, submitLabel,
}: {
  name: string; setName: (v: string) => void;
  slug: string; setSlug: (v: string) => void;
  modelId: string; setModelId: (v: string) => void;
  providerId: string; setProviderId: (v: string) => void;
  context: string; setContext: (v: string) => void;
  caps: string; setCaps: (v: string) => void;
  providers: AdminProvider[];
  t: (key: string, params?: Record<string, string | number>) => string;
  onSubmit: () => void;
  onCancel: () => void;
  submitLabel: string;
}) {
  return (
    <div style={formStyles.form}>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('models.nameLabel')}</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t('models.namePlaceholder')}
          style={formStyles.input}
          autoFocus
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('models.providerLabel')}</label>
        <select
          value={providerId}
          onChange={(e) => setProviderId(e.target.value)}
          style={formStyles.input}
        >
          <option value="" disabled>{t('models.selectProvider')}</option>
          {providers.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>Slug</label>
        <input
          type="text"
          value={slug}
          onChange={(e) => setSlug(e.target.value)}
          placeholder="gpt-4o"
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>Model ID</label>
        <input
          type="text"
          value={modelId}
          onChange={(e) => setModelId(e.target.value)}
          placeholder="gpt-4o"
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('models.contextLabel')}</label>
        <input
          type="number"
          value={context}
          onChange={(e) => setContext(e.target.value)}
          placeholder="128000"
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('models.capabilitiesLabel')}</label>
        <input
          type="text"
          value={caps}
          onChange={(e) => setCaps(e.target.value)}
          placeholder={t('models.capsPlaceholder')}
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.actions}>
        <button style={formStyles.cancelBtn} onClick={onCancel}>{t('common.cancel')}</button>
        <button style={formStyles.submitBtn} onClick={onSubmit} disabled={!name.trim() || !providerId}>
          {submitLabel}
        </button>
      </div>
    </div>
  );
}

const formStyles: Record<string, React.CSSProperties> = {
  form: { display: 'flex', flexDirection: 'column', gap: '16px' },
  field: { display: 'flex', flexDirection: 'column', gap: '6px' },
  label: {
    fontSize: '12px',
    fontWeight: '500',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
  input: {
    padding: '10px 12px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
    backgroundColor: '#FFFFFF',
    color: '#18181B',
    outline: 'none',
  },
  actions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '8px',
    marginTop: '8px',
  },
  cancelBtn: {
    padding: '8px 16px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF',
    fontSize: '12px',
    fontWeight: '500',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
  submitBtn: {
    padding: '8px 16px',
    borderRadius: '8px',
    border: 'none',
    backgroundColor: '#6366F1',
    fontSize: '12px',
    fontWeight: '500',
    color: '#FFFFFF',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
};

const styles: Record<string, React.CSSProperties> = {
  container: { maxWidth: '1200px' },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-end',
    marginBottom: '24px',
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
  addBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 14px',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    border: 'none',
    borderRadius: '10px',
    fontSize: '12px',
    fontWeight: '500',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
  tableCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    overflow: 'hidden',
  },
  table: { width: '100%', borderCollapse: 'collapse' },
  th: {
    padding: '12px 16px',
    textAlign: 'left',
    fontSize: '11px',
    fontWeight: '500',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    fontFamily: "'DM Sans', sans-serif",
    borderBottom: '1px solid #F5F5F4',
  },
  tr: {
    borderBottom: '1px solid #F5F5F4',
    transition: 'background 0.1s ease',
  },
  td: {
    padding: '14px 16px',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
  },
  modelName: { fontWeight: '500', color: '#18181B', display: 'block' },
  modelSlug: { fontSize: '11px', color: '#A1A1AA', display: 'block', marginTop: '1px' },
  providerBadge: {
    fontSize: '11px',
    fontWeight: '500',
    padding: '3px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  context: { color: '#71717A', fontSize: '12px' },
  caps: { display: 'flex', gap: '4px', flexWrap: 'wrap' },
  capTag: {
    fontSize: '10px',
    fontWeight: '500',
    color: '#71717A',
    backgroundColor: '#F5F5F4',
    padding: '2px 8px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  noCap: { color: '#A1A1AA' },
  status: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '12px',
    fontWeight: '500',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: { width: '5px', height: '5px', borderRadius: '50%', flexShrink: 0 },
  actions: { display: 'flex', gap: '8px', justifyContent: 'flex-end' },
  actionBtn: {
    padding: '5px 12px',
    backgroundColor: 'transparent',
    border: '1px solid #E7E5E4',
    borderRadius: '8px',
    fontSize: '11px',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.1s ease',
  },
};
