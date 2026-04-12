import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import { fetchProviders, createProvider, updateProvider, deleteProvider, type AdminProvider } from '../api/admin';

const providerColors: Record<string, string> = {
  openai: '#10A37F',
  anthropic: '#D97706',
  google: '#4285F4',
  deepseek: '#6366F1',
};

function getColor(slug: string, index: number): string {
  if (providerColors[slug]) return providerColors[slug];
  const fallback = ['#10A37F', '#D97706', '#4285F4', '#6366F1', '#EC4899', '#F59E0B'];
  return fallback[index % fallback.length];
}

export default function Providers() {
  const { t } = useI18n();
  const [providers, setProviders] = useState<AdminProvider[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [editProvider, setEditProvider] = useState<AdminProvider | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AdminProvider | null>(null);

  const [formName, setFormName] = useState('');
  const [formSlug, setFormSlug] = useState('');
  const [formApiUrl, setFormApiUrl] = useState('');
  const [formPriority, setFormPriority] = useState('1');

  const loadProviders = useCallback(() => {
    setLoading(true);
    fetchProviders()
      .then((res) => setProviders(res.data))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    loadProviders();
  }, [loadProviders]);

  const resetForm = () => {
    setFormName('');
    setFormSlug('');
    setFormApiUrl('');
    setFormPriority('1');
  };

  const openAddModal = () => {
    resetForm();
    setShowAddModal(true);
  };

  const openEditModal = (p: AdminProvider) => {
    setFormName(p.name);
    setFormSlug(p.slug);
    setFormApiUrl(p.api_base_url);
    setFormPriority(String(p.priority));
    setEditProvider(p);
  };

  const handleAddProvider = async () => {
    if (!formName.trim()) return;
    try {
      await createProvider({
        name: formName,
        slug: formSlug || formName.toLowerCase().replace(/\s+/g, '-'),
        api_base_url: formApiUrl || undefined,
        priority: parseInt(formPriority) || undefined,
      });
      setShowAddModal(false);
      loadProviders();
    } catch {
      // Error handling
    }
  };

  const handleEditProvider = async () => {
    if (!editProvider || !formName.trim()) return;
    try {
      await updateProvider(editProvider.id, {
        name: formName,
        slug: formSlug,
        api_base_url: formApiUrl,
        priority: parseInt(formPriority) || editProvider.priority,
      });
      setEditProvider(null);
      loadProviders();
    } catch {
      // Error handling
    }
  };

  const handleDeleteProvider = async () => {
    if (!deleteTarget) return;
    try {
      await deleteProvider(deleteTarget.id);
      setDeleteTarget(null);
      loadProviders();
    } catch {
      // Error handling
    }
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('providers.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? 'Loading...' : t('providers.subtitle')}
          </p>
        </div>
        <button style={styles.addBtn} onClick={openAddModal}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          {t('providers.addProvider')}
        </button>
      </header>

      <div style={styles.grid}>
        {providers.map((p, i) => {
          const color = getColor(p.slug, i);
          return (
            <div key={p.id} style={styles.card}>
              <div style={styles.cardTop}>
                <div style={{ ...styles.logo, backgroundColor: `${color}14`, color }}>
                  {p.name.charAt(0)}
                </div>
                <div style={styles.info}>
                  <h3 style={styles.name}>{p.name}</h3>
                  <span style={styles.slug}>{p.slug}</span>
                </div>
                <span style={{
                  ...styles.statusBadge,
                  color: p.is_active ? '#22C55E' : '#EF4444',
                  backgroundColor: p.is_active ? 'rgba(34, 197, 94, 0.08)' : 'rgba(239, 68, 68, 0.08)',
                }}>
                  <span style={{
                    ...styles.statusDot,
                    backgroundColor: p.is_active ? '#22C55E' : '#EF4444',
                  }} />
                  {p.is_active ? t('common.active') : t('common.inactive')}
                </span>
              </div>
              <div style={styles.stats}>
                <div style={styles.stat}>
                  <span style={styles.statValue}>{p.api_base_url.slice(0, 30)}{p.api_base_url.length > 30 ? '...' : ''}</span>
                  <span style={styles.statLabel}>API URL</span>
                </div>
                <div style={styles.statDivider} />
                <div style={styles.stat}>
                  <span style={styles.statValue}>#{p.priority}</span>
                  <span style={styles.statLabel}>{t('providers.priority')}</span>
                </div>
              </div>
              <div style={styles.cardActions}>
                <button style={styles.cardActionBtn} onClick={() => openEditModal(p)}>
                  {t('common.edit')}
                </button>
                <button
                  style={{ ...styles.cardActionBtn, color: '#EF4444', borderColor: 'rgba(239, 68, 68, 0.2)' }}
                  onClick={() => setDeleteTarget(p)}
                >
                  {t('common.delete')}
                </button>
              </div>
            </div>
          );
        })}
        {!loading && providers.length === 0 && (
          <div style={{ gridColumn: '1 / -1', textAlign: 'center', color: '#A1A1AA', padding: '40px', fontFamily: "'DM Sans', sans-serif", fontSize: '13px' }}>
            No providers yet
          </div>
        )}
      </div>

      <Modal open={showAddModal} onClose={() => setShowAddModal(false)} title={t('providers.addProvider')}>
        <ProviderForm
          name={formName} setName={setFormName}
          slug={formSlug} setSlug={setFormSlug}
          apiUrl={formApiUrl} setApiUrl={setFormApiUrl}
          priority={formPriority} setPriority={setFormPriority}
          t={t}
          onSubmit={handleAddProvider}
          onCancel={() => setShowAddModal(false)}
          submitLabel={t('providers.addProvider')}
        />
      </Modal>

      <Modal open={!!editProvider} onClose={() => setEditProvider(null)} title={t('providers.editProvider')}>
        <ProviderForm
          name={formName} setName={setFormName}
          slug={formSlug} setSlug={setFormSlug}
          apiUrl={formApiUrl} setApiUrl={setFormApiUrl}
          priority={formPriority} setPriority={setFormPriority}
          t={t}
          onSubmit={handleEditProvider}
          onCancel={() => setEditProvider(null)}
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
            <button style={{ ...formStyles.submitBtn, backgroundColor: '#EF4444' }} onClick={handleDeleteProvider}>
              {t('common.delete')}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function ProviderForm({
  name, setName, slug, setSlug, apiUrl, setApiUrl, priority, setPriority,
  t, onSubmit, onCancel, submitLabel,
}: {
  name: string; setName: (v: string) => void;
  slug: string; setSlug: (v: string) => void;
  apiUrl: string; setApiUrl: (v: string) => void;
  priority: string; setPriority: (v: string) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
  onSubmit: () => void;
  onCancel: () => void;
  submitLabel: string;
}) {
  return (
    <div style={formStyles.form}>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('providers.nameLabel')}</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t('providers.namePlaceholder')}
          style={formStyles.input}
          autoFocus
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('providers.slugLabel')}</label>
        <input
          type="text"
          value={slug}
          onChange={(e) => setSlug(e.target.value)}
          placeholder={t('providers.slugPlaceholder')}
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>API Base URL</label>
        <input
          type="text"
          value={apiUrl}
          onChange={(e) => setApiUrl(e.target.value)}
          placeholder="https://api.example.com/v1"
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('providers.priorityLabel')}</label>
        <input
          type="number"
          value={priority}
          onChange={(e) => setPriority(e.target.value)}
          min="1"
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.actions}>
        <button style={formStyles.cancelBtn} onClick={onCancel}>{t('common.cancel')}</button>
        <button style={formStyles.submitBtn} onClick={onSubmit} disabled={!name.trim()}>
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
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '14px',
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    padding: '20px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  cardTop: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  logo: {
    width: '40px',
    height: '40px',
    borderRadius: '10px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '16px',
    fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
    flexShrink: 0,
  },
  info: {
    flex: 1,
  },
  name: {
    fontSize: '14px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  slug: {
    fontSize: '12px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusBadge: {
    display: 'flex',
    alignItems: 'center',
    gap: '5px',
    fontSize: '11px',
    fontWeight: '500',
    padding: '4px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: {
    width: '5px',
    height: '5px',
    borderRadius: '50%',
  },
  stats: {
    display: 'flex',
    alignItems: 'center',
    paddingTop: '14px',
    borderTop: '1px solid #F5F5F4',
  },
  stat: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '2px',
  },
  statValue: {
    fontSize: '12px',
    fontWeight: '600',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  statLabel: {
    fontSize: '10px',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    fontFamily: "'DM Sans', sans-serif",
  },
  statDivider: {
    width: '1px',
    height: '24px',
    backgroundColor: '#F5F5F4',
  },
  cardActions: {
    display: 'flex',
    gap: '8px',
    justifyContent: 'flex-end',
    paddingTop: '4px',
  },
  cardActionBtn: {
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
