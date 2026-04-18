/**
 * @file BrowserAccounts - 浏览器账号管理页面
 * @description 动态展示服务商对应的浏览器账号，支持扫码认证
 * 管理员在服务商页面添加了多少个大模型服务商，就可以在此页面添加对应数量的浏览器账号
 */
import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import QrCodeModal from '../components/QrCodeModal';
import {
  fetchBrowserAccounts, createBrowserAccount, deleteBrowserAccount, startLogin,
  fetchProviders,
  type BrowserAccount, type QrCodeData, type AdminProvider,
} from '../api/admin';
import { getErrorMessage } from '../utils/errors';

// 服务商slug到浏览器provider的映射
const BROWSER_PROVIDER_MAP: Record<string, string> = {
  openai: 'chatgpt',
  anthropic: 'claude',
  deepseek: 'deepseek',
};

// 浏览器provider到颜色/品牌色的映射
const providerColors: Record<string, { bg: string; text: string; accent: string }> = {
  claude: { bg: '#FFF7ED', text: '#D97706', accent: '#FBBF24' },
  chatgpt: { bg: '#ECFDF5', text: '#10A37F', accent: '#34D399' },
  deepseek: { bg: '#EFF6FF', text: '#3B82F6', accent: '#60A5FA' },
};

function getProviderStyle(provider: string) {
  return providerColors[provider] || { bg: '#F5F5F4', text: '#71717A', accent: '#A1A1AA' };
}

// 服务商信息卡片
interface ProviderSlot {
  provider: AdminProvider;
  browserProvider: string;
  accounts: BrowserAccount[];
  maxAccounts: number;
}

export default function BrowserAccounts() {
  const { t } = useI18n();
  const [slots, setSlots] = useState<ProviderSlot[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<BrowserAccount | null>(null);
  const [qrModalData, setQrModalData] = useState<{ account: BrowserAccount; qrData: QrCodeData } | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedSlot, setSelectedSlot] = useState<ProviderSlot | null>(null);

  // 加载服务商和账号数据
  const loadData = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const [providersRes, accountsRes] = await Promise.all([
        fetchProviders(),
        fetchBrowserAccounts(),
      ]);

      const providers = providersRes.data;
      const accounts = accountsRes.data;

      // 为每个服务商创建账号槽位
      // 支持浏览器的服务商映射: openai->chatgpt, anthropic->claude, deepseek->deepseek
      const browserSlots: ProviderSlot[] = providers
        .filter(p => p.is_active)
        .map(provider => {
          const browserProvider = BROWSER_PROVIDER_MAP[provider.slug];
          if (!browserProvider) return null;

          const providerAccounts = accounts.filter(a => a.provider === browserProvider);
          return {
            provider,
            browserProvider,
            accounts: providerAccounts,
            maxAccounts: 1, // 默认每个服务商最多1个账号，后续可扩展
          };
        })
        .filter((s): s is ProviderSlot => s !== null);

      setSlots(browserSlots);
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleAddAccount = async (slot: ProviderSlot) => {
    try {
      await createBrowserAccount(slot.browserProvider);
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

  const handleQrGenerated = async (account: BrowserAccount) => {
    try {
      const loginData = await startLogin(account.id);
      const qrData: QrCodeData = {
        session_id: account.id,
        qr_code_data: '',
        code: loginData.code || '',
        expires_at: loginData.expires_at || '',
        auth_url: loginData.login_url,
      };
      setQrModalData({ account, qrData });
    } catch (err) {
      setError(getErrorMessage(err, t));
    }
  };

  const handleAuthSuccess = () => {
    setQrModalData(null);
    loadData();
  };

  const openAddModal = (slot: ProviderSlot) => {
    setSelectedSlot(slot);
    setShowAddModal(true);
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('browserAccounts.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? t('common.loading') : `已配置 ${slots.length} 个支持浏览器的服务商`}
          </p>
        </div>
      </header>

      {error && (
        <div style={styles.errorBanner}>
          {error}
          <button style={styles.errorClose} onClick={() => setError('')}>×</button>
        </div>
      )}

      {/* 服务商账号槽位网格 */}
      <div style={styles.slotsGrid}>
        {slots.map((slot) => {
          const style = getProviderStyle(slot.browserProvider);
          const hasAccounts = slot.accounts.length > 0;
          const account = hasAccounts ? slot.accounts[0] : null;
          const isPending = account?.status === 'pending';
          const isActive = account?.status === 'active';

          return (
            <div key={slot.provider.id} style={{ ...styles.slotCard, '--provider-accent': style.accent } as React.CSSProperties}>
              {/* 头部区域 */}
              <div style={{ ...styles.slotHeader, backgroundColor: style.bg }}>
                <div style={styles.slotHeaderLeft}>
                  <div style={{ ...styles.providerBadge, backgroundColor: `${style.text}18`, color: style.text }}>
                    {slot.provider.name.charAt(0)}
                  </div>
                  <div>
                    <h3 style={styles.slotTitle}>{slot.provider.name}</h3>
                    <span style={{ ...styles.slotSubtitle, color: style.text }}>
                      {slot.browserProvider.toUpperCase()}
                    </span>
                  </div>
                </div>
                <div style={{ ...styles.statusBadge, color: isActive ? '#22C55E' : isPending ? '#F59E0B' : '#A1A1AA', backgroundColor: isActive ? 'rgba(34, 197, 94, 0.08)' : isPending ? 'rgba(245, 158, 11, 0.08)' : 'rgba(161, 161, 170, 0.08)' }}>
                  <span style={{ ...styles.statusDot, backgroundColor: isActive ? '#22C55E' : isPending ? '#F59E0B' : '#A1A1AA' }} />
                  {account ? t(`browserAccounts.status.${account.status}`) : '未配置'}
                </div>
              </div>

              {/* 账号信息区域 */}
              <div style={styles.slotBody}>
                {account ? (
                  <>
                    <div style={styles.accountInfo}>
                      <div style={styles.infoRow}>
                        <span style={styles.infoLabel}>{t('browserAccounts.email')}</span>
                        <span style={styles.infoValue}>{account.email || '-'}</span>
                      </div>
                      <div style={styles.infoRow}>
                        <span style={styles.infoLabel}>{t('browserAccounts.requests')}</span>
                        <span style={styles.infoValue}>{account.request_count.toLocaleString()}</span>
                      </div>
                      <div style={styles.infoRow}>
                        <span style={styles.infoLabel}>{t('browserAccounts.lastUsed')}</span>
                        <span style={styles.infoValue}>
                          {account.last_used_at ? new Date(account.last_used_at).toLocaleDateString() : '-'}
                        </span>
                      </div>
                    </div>

                    <div style={styles.slotActions}>
                      {isPending && (
                        <button
                          style={{ ...styles.actionBtn, backgroundColor: style.text, color: '#fff', flex: 1 }}
                          onClick={() => handleQrGenerated(account)}
                        >
                          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                            <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
                            <rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" />
                          </svg>
                          扫码认证
                        </button>
                      )}
                      <button
                        style={{ ...styles.actionBtn, color: '#EF4444' }}
                        onClick={() => setDeleteTarget(account)}
                      >
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                        </svg>
                      </button>
                    </div>
                  </>
                ) : (
                  <div style={styles.emptySlot}>
                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke={style.text} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" style={{ opacity: 0.4 }}>
                      <circle cx="12" cy="12" r="10" />
                      <line x1="12" y1="8" x2="12" y2="16" /><line x1="8" y1="12" x2="16" y2="12" />
                    </svg>
                    <p style={{ ...styles.emptySlotText, color: style.text }}>点击添加浏览器账号</p>
                    <button
                      style={{ ...styles.addAccountBtn, backgroundColor: style.text }}
                      onClick={() => openAddModal(slot)}
                    >
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
                      </svg>
                      添加账号
                    </button>
                  </div>
                )}
              </div>
            </div>
          );
        })}

        {!loading && slots.length === 0 && (
          <div style={styles.emptyState}>
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
              <circle cx="9" cy="7" r="4" />
              <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
              <path d="M16 3.13a4 4 0 0 1 0 7.75" />
            </svg>
            <h3 style={styles.emptyTitle}>暂无可用服务商</h3>
            <p style={styles.emptyDesc}>
              请先在{` `}
              <a href="#/providers" style={{ color: '#6366F1', textDecoration: 'none', fontWeight: 500 }}>服务商页面</a>
              {` `}添加至少一个大模型服务商
            </p>
          </div>
        )}
      </div>

      {/* 添加账号确认弹窗 */}
      <Modal open={showAddModal} onClose={() => setShowAddModal(false)} title="添加浏览器账号" width={400}>
        {selectedSlot && (
          <div style={addModalStyles.content}>
            <div style={addModalStyles.providerInfo}>
              <div style={{ ...addModalStyles.providerIcon, backgroundColor: getProviderStyle(selectedSlot.browserProvider).bg, color: getProviderStyle(selectedSlot.browserProvider).text }}>
                {selectedSlot.provider.name.charAt(0)}
              </div>
              <div>
                <h4 style={addModalStyles.providerName}>{selectedSlot.provider.name}</h4>
                <span style={addModalStyles.providerSlug}>{selectedSlot.browserProvider.toUpperCase()}</span>
              </div>
            </div>

            <p style={addModalStyles.desc}>
              将为该服务商创建一个浏览器账号。创建后需要通过扫码完成认证才能使用。
            </p>

            <div style={addModalStyles.actions}>
              <button style={addModalStyles.cancelBtn} onClick={() => setShowAddModal(false)}>
                取消
              </button>
              <button
                style={{ ...addModalStyles.confirmBtn, backgroundColor: getProviderStyle(selectedSlot.browserProvider).text }}
                onClick={() => {
                  setShowAddModal(false);
                  handleAddAccount(selectedSlot);
                }}
              >
                确认添加
              </button>
            </div>
          </div>
        )}
      </Modal>

      {/* 删除确认弹窗 */}
      <Modal open={!!deleteTarget} onClose={() => setDeleteTarget(null)} title="删除账号" width={380}>
        <div style={deleteModalStyles.content}>
          <div style={deleteModalStyles.iconWrapper}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" />
            </svg>
          </div>
          <p style={deleteModalStyles.message}>确定要删除此浏览器账号吗？此操作不可撤销。</p>
          <div style={deleteModalStyles.actions}>
            <button style={deleteModalStyles.cancelBtn} onClick={() => setDeleteTarget(null)}>
              取消
            </button>
            <button style={deleteModalStyles.deleteBtn} onClick={handleDelete}>
              删除
            </button>
          </div>
        </div>
      </Modal>

      {/* QR 码弹窗 */}
      {qrModalData && (
        <QrCodeModal
          account={qrModalData.account}
          qrData={qrModalData.qrData}
          onClose={() => setQrModalData(null)}
          onSuccess={handleAuthSuccess}
        />
      )}
    </div>
  );
}

// 添加账号弹窗样式
const addModalStyles: Record<string, React.CSSProperties> = {
  content: { display: 'flex', flexDirection: 'column', gap: '20px' },
  providerInfo: { display: 'flex', alignItems: 'center', gap: '12px' },
  providerIcon: {
    width: '44px', height: '44px', borderRadius: '12px',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    fontSize: '18px', fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
  },
  providerName: { fontSize: '15px', fontWeight: '600', color: '#18181B', margin: 0, fontFamily: "'DM Sans', sans-serif" },
  providerSlug: { fontSize: '11px', color: '#A1A1AA', fontFamily: "'DM Sans', sans-serif", letterSpacing: '0.04em' },
  desc: { fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif", lineHeight: 1.5 },
  actions: { display: 'flex', justifyContent: 'flex-end', gap: '10px', marginTop: '4px' },
  cancelBtn: {
    padding: '9px 18px', borderRadius: '10px', border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF', fontSize: '13px', fontWeight: '500', color: '#71717A',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
  confirmBtn: {
    padding: '9px 18px', borderRadius: '10px', border: 'none',
    fontSize: '13px', fontWeight: '500', color: '#FFFFFF',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
};

// 删除确认弹窗样式
const deleteModalStyles: Record<string, React.CSSProperties> = {
  content: { display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '16px', padding: '8px 0' },
  iconWrapper: {
    width: '56px', height: '56px', borderRadius: '50%',
    backgroundColor: 'rgba(239, 68, 68, 0.08)',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
  },
  message: { fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif", textAlign: 'center', lineHeight: 1.5 },
  actions: { display: 'flex', gap: '10px', width: '100%' },
  cancelBtn: {
    flex: 1, padding: '10px', borderRadius: '10px', border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF', fontSize: '13px', fontWeight: '500', color: '#71717A',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
  deleteBtn: {
    flex: 1, padding: '10px', borderRadius: '10px', border: 'none',
    backgroundColor: '#EF4444', fontSize: '13px', fontWeight: '500', color: '#FFFFFF',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
};

const styles: Record<string, React.CSSProperties> = {
  container: { maxWidth: '1100px' },
  header: {
    display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end',
    marginBottom: '28px',
  },
  pageTitle: {
    fontSize: '26px', fontWeight: '700', color: '#18181B', margin: 0,
    fontFamily: "'Instrument Sans', sans-serif", letterSpacing: '-0.02em',
  },
  pageSubtitle: {
    fontSize: '13px', color: '#71717A', marginTop: '4px',
    fontFamily: "'DM Sans', sans-serif",
  },
  slotsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
    gap: '20px',
  },
  slotCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '18px',
    boxShadow: '0 2px 8px rgba(0,0,0,0.04), 0 0 0 1px rgba(0,0,0,0.02)',
    overflow: 'hidden',
    transition: 'box-shadow 0.2s ease, transform 0.2s ease',
  },
  slotHeader: {
    display: 'flex', justifyContent: 'space-between', alignItems: 'center',
    padding: '18px 20px',
  },
  slotHeaderLeft: { display: 'flex', alignItems: 'center', gap: '12px' },
  providerBadge: {
    width: '42px', height: '42px', borderRadius: '12px',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    fontSize: '17px', fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
  },
  slotTitle: {
    fontSize: '15px', fontWeight: '600', color: '#18181B', margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  slotSubtitle: {
    fontSize: '10px', fontWeight: '600', letterSpacing: '0.08em',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusBadge: {
    display: 'inline-flex', alignItems: 'center', gap: '5px',
    fontSize: '11px', fontWeight: '500', padding: '4px 10px', borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: { width: '5px', height: '5px', borderRadius: '50%' },
  slotBody: { padding: '20px' },
  accountInfo: { display: 'flex', flexDirection: 'column', gap: '10px' },
  infoRow: { display: 'flex', justifyContent: 'space-between', alignItems: 'center' },
  infoLabel: { fontSize: '12px', color: '#A1A1AA', fontFamily: "'DM Sans', sans-serif" },
  infoValue: { fontSize: '12px', color: '#18181B', fontFamily: "'DM Sans', sans-serif" },
  slotActions: { display: 'flex', gap: '8px', marginTop: '14px', paddingTop: '14px', borderTop: '1px solid #F5F5F4' },
  actionBtn: {
    display: 'flex', alignItems: 'center', gap: '6px',
    padding: '7px 13px',
    backgroundColor: 'transparent',
    border: '1px solid #E7E5E4',
    borderRadius: '8px',
    fontSize: '11px', fontWeight: '500', color: '#71717A',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.15s ease',
  },
  emptySlot: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    gap: '10px', padding: '20px 0',
  },
  emptySlotText: {
    fontSize: '12px', margin: 0,
    fontFamily: "'DM Sans', sans-serif", opacity: 0.7,
  },
  addAccountBtn: {
    display: 'flex', alignItems: 'center', gap: '6px',
    padding: '8px 16px', border: 'none', borderRadius: '10px',
    fontSize: '12px', fontWeight: '500', color: '#FFFFFF',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
    marginTop: '4px',
  },
  emptyState: {
    gridColumn: '1 / -1',
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    justifyContent: 'center', padding: '72px 20px',
    backgroundColor: '#FFFFFF', borderRadius: '18px',
    boxShadow: '0 2px 8px rgba(0,0,0,0.04)',
  },
  emptyTitle: {
    fontSize: '16px', fontWeight: '600', color: '#71717A',
    margin: '16px 0 6px', fontFamily: "'DM Sans', sans-serif",
  },
  emptyDesc: {
    fontSize: '13px', color: '#A1A1AA', margin: 0,
    fontFamily: "'DM Sans', sans-serif", textAlign: 'center',
  },
  errorBanner: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 16px',
    backgroundColor: 'rgba(239, 68, 68, 0.08)',
    border: '1px solid rgba(239, 68, 68, 0.2)',
    borderRadius: '10px',
    marginBottom: '20px',
    fontSize: '13px',
    color: '#EF4444',
    fontFamily: "'DM Sans', sans-serif",
  },
  errorClose: {
    background: 'none',
    border: 'none',
    fontSize: '18px',
    color: '#EF4444',
    cursor: 'pointer',
    padding: '0 4px',
    lineHeight: 1,
  },
};