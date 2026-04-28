import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import LoginModal from '../components/LoginModal';
import {
  fetchBrowserAccounts, createBrowserAccount, deleteBrowserAccount,
  fetchProviders,
  type BrowserAccount, type AdminProvider,
} from '../api/admin';
import { getErrorMessage } from '../utils/errors';

const BROWSER_PROVIDER_MAP: Record<string, string> = {
  openai: 'chatgpt',
  anthropic: 'claude',
  deepseek: 'deepseek',
};

const providerColors: Record<string, { bg: string; text: string; accent: string }> = {
  claude: { bg: '#FFF7ED', text: '#D97706', accent: '#FBBF24' },
  chatgpt: { bg: '#ECFDF5', text: '#10A37F', accent: '#34D399' },
  deepseek: { bg: '#EFF6FF', text: '#3B82F6', accent: '#60A5FA' },
};

function getProviderStyle(provider: string) {
  return providerColors[provider] || { bg: '#F5F5F4', text: '#71717A', accent: '#A1A1AA' };
}

export default function BrowserAccounts() {
  const { t } = useI18n();
  const [providers, setProviders] = useState<AdminProvider[]>([]);
  const [accounts, setAccounts] = useState<BrowserAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<BrowserAccount | null>(null);
  const [loginModalData, setLoginModalData] = useState<BrowserAccount | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [addModalProvider, setAddModalProvider] = useState<AdminProvider | null>(null);
  const [newAccountName, setNewAccountName] = useState('');

  const loadData = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const [providersRes, accountsRes] = await Promise.all([
        fetchProviders(),
        fetchBrowserAccounts(),
      ]);
      setProviders(providersRes.data.filter(p => p.is_active));
      setAccounts(accountsRes.data);
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => { loadData(); }, [loadData]);
  useEffect(() => {
    const interval = setInterval(() => loadData(), 30000);
    return () => clearInterval(interval);
  }, [loadData]);

  const handleAddAccount = async () => {
    if (!addModalProvider) return;
    if (!newAccountName.trim()) {
      setError(t('browserAccounts.nameRequired'));
      return;
    }
    const browserProvider = BROWSER_PROVIDER_MAP[addModalProvider.slug];
    if (!browserProvider) return;
    try {
      await createBrowserAccount(browserProvider, newAccountName.trim() || undefined);
      setShowAddModal(false);
      setNewAccountName('');
      loadData();
    } catch (err) {
      setError(getErrorMessage(err, t));
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteBrowserAccount(deleteTarget.id);
      setDeleteTarget(null);
      loadData();
    } catch (err) {
      setError(getErrorMessage(err, t));
    }
  };

  const browserProviders = providers
    .map(p => ({ provider: p, browserProvider: BROWSER_PROVIDER_MAP[p.slug] }))
    .filter(s => s.browserProvider);

  return (
    <div style={s.container}>
      <header style={s.header}>
        <div>
          <h1 style={s.pageTitle}>{t('browserAccounts.title')}</h1>
          <p style={s.pageSubtitle}>
            {loading ? t('common.loading') : t('browserAccounts.subtitle', { count: accounts.length.toString() })}
          </p>
        </div>
      </header>

      {error && (
        <div style={s.errorBanner}>
          {error}
          <button style={s.errorClose} onClick={() => setError('')}>&times;</button>
        </div>
      )}

      {browserProviders.length === 0 && !loading && (
        <div style={s.emptyState}>
          <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" />
            <path d="M23 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" />
          </svg>
          <h3 style={s.emptyTitle}>{t('browserAccounts.noProviders')}</h3>
          <p style={s.emptyDesc}>
            <a href="#/providers" style={{ color: '#6366F1', textDecoration: 'none', fontWeight: 500 }}>
              {t('browserAccounts.goProviders')}
            </a>
          </p>
        </div>
      )}

      {browserProviders.map(({ provider, browserProvider }) => {
        const style = getProviderStyle(browserProvider);
        const providerAccounts = accounts.filter(a => a.provider === browserProvider);

        return (
          <section key={provider.id} style={s.providerSection}>
            <div style={s.providerSectionHeader}>
              <div style={{ ...s.providerBadge, background: `${style.text}16`, color: style.text }}>
                {provider.name.charAt(0)}
              </div>
              <div>
                <h2 style={s.providerName}>{provider.name}</h2>
                <span style={{ ...s.providerSlug, color: style.text }}>{browserProvider.toUpperCase()}</span>
              </div>
              <div style={{ flex: 1 }} />
              <span style={s.accountCount}>
                {providerAccounts.length} {t('browserAccounts.accountCountLabel')}
              </span>
            </div>

            <div style={s.accountsGrid}>
              {providerAccounts.map(acc => {
                const isActive = acc.status === 'active';
                const isPending = acc.status === 'pending';

                return (
                  <div key={acc.id} style={s.accountCard}>
                    <div style={s.cardTop}>
                      <div style={s.cardHeaderRow}>
                        <div style={s.cardNameBlock}>
                          <h4 style={s.cardName}>{acc.name || t('browserAccounts.unnamed')}</h4>
                          {acc.email && <span style={s.cardEmail}>{acc.email}</span>}
                        </div>
                        <span style={{
                          ...s.statusPill,
                          color: isActive ? '#16A34A' : isPending ? '#D97706' : '#A1A1AA',
                          background: isActive ? '#F0FDF4' : isPending ? '#FFFBEB' : '#FAFAFA',
                        }}>
                          <span style={{ ...s.statusDot, background: isActive ? '#16A34A' : isPending ? '#D97706' : '#A1A1AA' }} />
                          {t(`browserAccounts.status.${acc.status}`)}
                        </span>
                      </div>

                      <div style={s.statsRow}>
                        <span style={s.statItem}>
                          <span style={s.statValue}>{acc.request_count.toLocaleString()}</span>
                          <span style={s.statLabel}>{t('browserAccounts.requests')}</span>
                        </span>
                        <span style={s.statItem}>
                          <span style={s.statValue}>
                            {acc.last_used_at ? new Date(acc.last_used_at).toLocaleDateString() : '-'}
                          </span>
                          <span style={s.statLabel}>{t('browserAccounts.lastUsed')}</span>
                        </span>
                      </div>
                    </div>

                    <div style={s.cardActions}>
                      <button
                        style={{ ...s.cookieBtn, background: style.text }}
                        onClick={() => setLoginModalData(acc)}
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <circle cx="12" cy="12" r="10" /><path d="M8 12l3 3 5-5" />
                        </svg>
                        {t('browserAccounts.cookieInject')}
                      </button>
                      <button
                        style={s.deleteBtn}
                        onClick={() => setDeleteTarget(acc)}
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                        </svg>
                      </button>
                    </div>
                  </div>
                );
              })}

              <div
                style={s.addCard}
                onClick={() => { setAddModalProvider(provider); setShowAddModal(true); }}
              >
                <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke={style.text} strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" style={{ opacity: 0.5 }}>
                  <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="16" /><line x1="8" y1="12" x2="16" y2="12" />
                </svg>
                <span style={{ ...s.addCardText, color: style.text }}>
                  {t('browserAccounts.addAccount')}
                </span>
              </div>
            </div>
          </section>
        );
      })}

      <Modal open={showAddModal} onClose={() => { setShowAddModal(false); setNewAccountName(''); }} title={t('browserAccounts.addAccount')} width={400}>
        {addModalProvider && (() => {
          const bp = BROWSER_PROVIDER_MAP[addModalProvider.slug] ?? '';
          const style = getProviderStyle(bp);
          return (
            <div style={addS.content}>
              <div style={addS.field}>
                <label style={addS.label}>{t('browserAccounts.accountName')}</label>
                <input
                  style={{ ...addS.input, borderColor: style.text + '30' }}
                  value={newAccountName}
                  onChange={e => setNewAccountName(e.target.value)}
                  placeholder={t('browserAccounts.accountNamePlaceholder')}
                  autoFocus
                  onKeyDown={e => { if (e.key === 'Enter') handleAddAccount(); }}
                />
              </div>
              <div style={addS.providerLine}>
                <div style={{ ...addS.providerIcon, background: style.bg, color: style.text }}>
                  {addModalProvider.name.charAt(0)}
                </div>
                <span style={addS.providerText}>
                  {addModalProvider.name} &middot; {bp.toUpperCase()}
                </span>
              </div>
              <div style={addS.actions}>
                <button style={addS.cancel} onClick={() => { setShowAddModal(false); setNewAccountName(''); }}>
                  {t('common.cancel')}
                </button>
                <button style={{ ...addS.confirm, background: style.text }} onClick={handleAddAccount}>
                  {t('common.confirm')}
                </button>
              </div>
            </div>
          );
        })()}
      </Modal>

      <Modal open={!!deleteTarget} onClose={() => setDeleteTarget(null)} title={t('browserAccounts.deleteConfirmTitle')} width={380}>
        <div style={delS.content}>
          <div style={delS.iconWrap}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" />
            </svg>
          </div>
          <p style={delS.msg}>{t('browserAccounts.deleteConfirm')}</p>
          <div style={delS.actions}>
            <button style={delS.cancel} onClick={() => setDeleteTarget(null)}>{t('common.cancel')}</button>
            <button style={delS.ok} onClick={handleDelete}>{t('common.delete')}</button>
          </div>
        </div>
      </Modal>

      {loginModalData && (
        <LoginModal account={loginModalData} onClose={() => setLoginModalData(null)} onSuccess={() => { setLoginModalData(null); loadData(); }} />
      )}
    </div>
  );
}

const addS: Record<string, React.CSSProperties> = {
  content: { display: 'flex', flexDirection: 'column', gap: '16px' },
  field: { display: 'flex', flexDirection: 'column', gap: '6px' },
  label: { fontSize: '13px', fontWeight: '600', color: '#18181B', fontFamily: "'DM Sans', sans-serif" },
  input: {
    padding: '12px 14px', borderRadius: '10px', border: '1.5px solid', fontSize: '14px',
    fontFamily: "'DM Sans', sans-serif", color: '#18181B', outline: 'none',
    background: '#F9FAFB',
  },
  providerLine: { display: 'flex', alignItems: 'center', gap: '10px' },
  providerIcon: {
    width: '34px', height: '34px', borderRadius: '10px', display: 'flex',
    alignItems: 'center', justifyContent: 'center', fontSize: '15px', fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
  },
  providerText: { fontSize: '13px', color: '#71717A', fontFamily: "'DM Sans', sans-serif" },
  actions: { display: 'flex', justifyContent: 'flex-end', gap: '10px', marginTop: '6px' },
  cancel: {
    padding: '9px 18px', borderRadius: '10px', border: '1px solid #E7E5E4',
    background: '#fff', fontSize: '13px', fontWeight: '500', color: '#71717A',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
  confirm: {
    padding: '9px 18px', borderRadius: '10px', border: 'none',
    fontSize: '13px', fontWeight: '600', color: '#fff', cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
};

const delS: Record<string, React.CSSProperties> = {
  content: { display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '16px', padding: '8px 0' },
  iconWrap: { width: '56px', height: '56px', borderRadius: '50%', background: '#FEF2F2', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  msg: { fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif", textAlign: 'center' },
  actions: { display: 'flex', gap: '10px', width: '100%' },
  cancel: { flex: 1, padding: '10px', borderRadius: '10px', border: '1px solid #E7E5E4', background: '#fff', fontSize: '13px', fontWeight: '500', color: '#71717A', cursor: 'pointer', fontFamily: "'DM Sans', sans-serif" },
  ok: { flex: 1, padding: '10px', borderRadius: '10px', border: 'none', background: '#EF4444', fontSize: '13px', fontWeight: '500', color: '#fff', cursor: 'pointer', fontFamily: "'DM Sans', sans-serif" },
};

const s: Record<string, React.CSSProperties> = {
  container: { maxWidth: '1200px' },
  header: { display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: '28px' },
  pageTitle: { fontSize: '26px', fontWeight: '700', color: '#18181B', margin: 0, fontFamily: "'Instrument Sans', sans-serif", letterSpacing: '-0.02em' },
  pageSubtitle: { fontSize: '13px', color: '#71717A', marginTop: '4px', fontFamily: "'DM Sans', sans-serif" },
  errorBanner: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 16px', background: 'rgba(239,68,68,0.08)', border: '1px solid rgba(239,68,68,0.2)', borderRadius: '10px', marginBottom: '20px', fontSize: '13px', color: '#EF4444', fontFamily: "'DM Sans', sans-serif" },
  errorClose: { background: 'none', border: 'none', fontSize: '18px', color: '#EF4444', cursor: 'pointer', padding: '0 4px', lineHeight: 1 },
  emptyState: { display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '72px 20px', background: '#fff', borderRadius: '18px', boxShadow: '0 2px 8px rgba(0,0,0,0.04)' },
  emptyTitle: { fontSize: '16px', fontWeight: '600', color: '#71717A', margin: '16px 0 6px', fontFamily: "'DM Sans', sans-serif" },
  emptyDesc: { fontSize: '13px', color: '#A1A1AA', margin: 0, fontFamily: "'DM Sans', sans-serif", textAlign: 'center' },

  providerSection: { marginBottom: '36px' },
  providerSectionHeader: { display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '16px' },
  providerBadge: { width: '40px', height: '40px', borderRadius: '12px', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '16px', fontWeight: '700', fontFamily: "'Instrument Sans', sans-serif" },
  providerName: { fontSize: '17px', fontWeight: '700', color: '#18181B', margin: 0, fontFamily: "'DM Sans', sans-serif", letterSpacing: '-0.01em' },
  providerSlug: { fontSize: '10px', fontWeight: '600', fontFamily: "'DM Sans', sans-serif", letterSpacing: '0.06em' },
  accountCount: { fontSize: '12px', fontWeight: '500', color: '#A1A1AA', fontFamily: "'DM Sans', sans-serif", padding: '4px 10px', background: '#F5F5F4', borderRadius: '9999px' },

  accountsGrid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(290px, 1fr))', gap: '14px' },

  accountCard: { background: '#fff', borderRadius: '16px', border: '1px solid #F0EDED', overflow: 'hidden', display: 'flex', flexDirection: 'column', transition: 'box-shadow 0.15s ease, border-color 0.15s ease' },
  cardTop: { padding: '16px 18px 12px' },
  cardHeaderRow: { display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', gap: '10px', marginBottom: '12px' },
  cardNameBlock: { flex: 1, minWidth: 0 },
  cardName: { fontSize: '15px', fontWeight: '600', color: '#18181B', margin: 0, fontFamily: "'DM Sans', sans-serif", letterSpacing: '-0.01em', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  cardEmail: { fontSize: '11px', color: '#A1A1AA', fontFamily: "'DM Sans', sans-serif", marginTop: '2px', display: 'block', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  statusPill: { display: 'inline-flex', alignItems: 'center', gap: '5px', fontSize: '11px', fontWeight: '600', padding: '3px 10px', borderRadius: '9999px', fontFamily: "'DM Sans', sans-serif", flexShrink: 0 },
  statusDot: { width: '5px', height: '5px', borderRadius: '50%' },

  statsRow: { display: 'flex', gap: '20px' },
  statItem: { display: 'flex', flexDirection: 'column', gap: '1px' },
  statValue: { fontSize: '13px', fontWeight: '600', color: '#18181B', fontFamily: "'DM Sans', sans-serif" },
  statLabel: { fontSize: '10px', color: '#A1A1AA', fontFamily: "'DM Sans', sans-serif" },

  cardActions: { display: 'flex', gap: '0', borderTop: '1px solid #F5F5F4' },
  cookieBtn: { flex: 1, padding: '10px 14px', border: 'none', fontSize: '12px', fontWeight: '600', color: '#fff', cursor: 'pointer', fontFamily: "'DM Sans', sans-serif", display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '6px', transition: 'opacity 0.15s' },
  deleteBtn: { padding: '10px 14px', background: 'transparent', border: 'none', color: '#EF4444', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center' },

  addCard: { background: '#FAFAFA', borderRadius: '16px', border: '2px dashed #E7E5E4', minHeight: '140px', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: '8px', cursor: 'pointer', transition: 'border-color 0.15s, background 0.15s' },
  addCardText: { fontSize: '13px', fontWeight: '600', fontFamily: "'DM Sans', sans-serif", opacity: 0.7 },
};
