/**
 * @file BrowserAccounts - 浏览器账号管理页面
 * 管理 Claude.ai 和 ChatGPT 的浏览器自动化登录账号
 * 支持扫码认证，展示账号状态和用量统计
 */
import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import QrCodeModal from '../components/QrCodeModal';
import {
  fetchBrowserAccounts, createBrowserAccount, deleteBrowserAccount, startLogin,
  type BrowserAccount, type QrCodeData,
} from '../api/admin';

// 提供商颜色映射
const providerColors: Record<string, string> = {
  deepseek: '#0068FF',
  claude: '#D97706',
  chatgpt: '#10A37F',
};

function getProviderColor(provider: string): string {
  return providerColors[provider] || '#A1A1AA';
}

/**
 * BrowserAccounts - 浏览器账号管理主组件
 * @description 获取账号列表，支持添加（扫码认证）、删除账号
 */
export default function BrowserAccounts() {
  const { t } = useI18n();
  const [accounts, setAccounts] = useState<BrowserAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [deleteTarget, setDeleteTarget] = useState<BrowserAccount | null>(null);
  // QR 码弹窗数据
  const [qrModalData, setQrModalData] = useState<{ account: BrowserAccount; qrData: QrCodeData } | null>(null);

  // 加载账号列表
  const loadAccounts = useCallback(() => {
    setLoading(true);
    fetchBrowserAccounts()
      .then((res) => setAccounts(res.data))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    loadAccounts();
  }, [loadAccounts]);

  const handleAddAccount = async (provider: string) => {
    try {
      await createBrowserAccount(provider);
      loadAccounts();
    } catch {
      // Error
    }
  };

  // 删除账号
  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteBrowserAccount(deleteTarget.id);
      setDeleteTarget(null);
      loadAccounts();
    } catch {
      // Error
    }
  };

  // 启动登录流程
  const handleQrGenerated = async (account: BrowserAccount) => {
    try {
      const loginData = await startLogin(account.id);
      // 转换为 QrCodeData 格式以保持兼容性
      const qrData: QrCodeData = {
        session_id: account.id,
        qr_code_data: '',  // 前端不再需要 base64 图片，直接用 URL
        code: loginData.code || '',
        expires_at: loginData.expires_at || '',
        auth_url: loginData.login_url,
      };
      setQrModalData({ account, qrData });
    } catch (err) {
      console.error('Failed to start login:', err);
    }
  };

  const handleAuthSuccess = () => {
    setQrModalData(null);
    loadAccounts();
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('browserAccounts.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? 'Loading...' : t('browserAccounts.subtitle')}
          </p>
        </div>
        <div style={styles.headerActions}>
          <button
            style={{ ...styles.addBtn, backgroundColor: '#D97706' }}
            onClick={() => handleAddAccount('claude')}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            {t('browserAccounts.addClaude')}
          </button>
          <button
            style={{ ...styles.addBtn, backgroundColor: '#10A37F' }}
            onClick={() => handleAddAccount('chatgpt')}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            {t('browserAccounts.addChatGPT')}
          </button>
          <button
            style={{ ...styles.addBtn, backgroundColor: '#0068FF' }}
            onClick={() => handleAddAccount('deepseek')}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            DeepSeek
          </button>
        </div>
      </header>

      {/* Accounts Grid */}
      <div style={styles.grid}>
        {accounts.map((account) => {
          const color = getProviderColor(account.provider);
          const isPending = account.status === 'pending';
          const isActive = account.status === 'active';
          const isError = account.status === 'error';

          return (
            <div key={account.id} style={styles.card}>
              <div style={styles.cardHeader}>
                <div style={styles.providerInfo}>
                  <div style={{ ...styles.providerDot, backgroundColor: `${color}18`, color }}>
                    {account.provider === 'claude' ? 'C' : 'C'}
                  </div>
                  <span style={styles.providerName}>{account.provider.toUpperCase()}</span>
                </div>
                <span style={{
                  ...styles.statusBadge,
                  color: isActive ? '#22C55E' : isError ? '#EF4444' : '#F59E0B',
                  backgroundColor: isActive ? 'rgba(34, 197, 94, 0.08)' : isError ? 'rgba(239, 68, 68, 0.08)' : 'rgba(245, 158, 11, 0.08)',
                }}>
                  <span style={{
                    ...styles.statusDot,
                    backgroundColor: isActive ? '#22C55E' : isError ? '#EF4444' : '#F59E0B',
                  }} />
                  {t(`browserAccounts.status.${account.status}`)}
                </span>
              </div>

              <div style={styles.cardBody}>
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
                    {account.last_used_at
                      ? new Date(account.last_used_at).toLocaleDateString()
                      : '-'}
                  </span>
                </div>
              </div>

              <div style={styles.cardActions}>
                {isPending && (
                  <button
                    style={{ ...styles.actionBtn, backgroundColor: '#6366F1', color: '#fff' }}
                    onClick={() => handleQrGenerated(account)}
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
                      <rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" />
                    </svg>
                    {t('browserAccounts.generateQr')}
                  </button>
                )}
                <button
                  style={{ ...styles.actionBtn, color: '#EF4444' }}
                  onClick={() => setDeleteTarget(account)}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  </svg>
                  {t('common.delete')}
                </button>
              </div>
            </div>
          );
        })}

        {!loading && accounts.length === 0 && (
          <div style={styles.emptyState}>
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" />
            </svg>
            <h3 style={styles.emptyTitle}>{t('browserAccounts.noAccounts')}</h3>
            <p style={styles.emptyDesc}>{t('browserAccounts.noAccountsDesc')}</p>
          </div>
        )}
      </div>

      {/* Delete Confirmation */}
      <Modal open={!!deleteTarget} onClose={() => setDeleteTarget(null)} title={t('common.delete')} width={380}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <p style={{ fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif" }}>
            {t('browserAccounts.deleteConfirm')}
          </p>
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
            <button style={formStyles.cancelBtn} onClick={() => setDeleteTarget(null)}>
              {t('common.cancel')}
            </button>
            <button style={{ ...formStyles.submitBtn, backgroundColor: '#EF4444' }} onClick={handleDelete}>
              {t('common.delete')}
            </button>
          </div>
        </div>
      </Modal>

      {/* QR Code Modal */}
      {qrModalData && (
        <QrCodeModal
          account={qrModalData.account}
          onClose={() => setQrModalData(null)}
          onSuccess={handleAuthSuccess}
        />
      )}
    </div>
  );
}

const formStyles: Record<string, React.CSSProperties> = {
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
  headerActions: {
    display: 'flex',
    gap: '8px',
  },
  addBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 14px',
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
    gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))',
    gap: '16px',
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    padding: '20px',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  providerInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  providerDot: {
    width: '36px',
    height: '36px',
    borderRadius: '10px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '14px',
    fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
  },
  providerName: {
    fontSize: '14px',
    fontWeight: '600',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
    letterSpacing: '0.02em',
  },
  statusBadge: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '5px',
    fontSize: '11px',
    fontWeight: '500',
    padding: '4px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
    textTransform: 'capitalize',
  },
  statusDot: {
    width: '5px',
    height: '5px',
    borderRadius: '50%',
  },
  cardBody: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  infoRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  infoLabel: {
    fontSize: '12px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  infoValue: {
    fontSize: '12px',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  cardActions: {
    display: 'flex',
    gap: '8px',
    paddingTop: '8px',
    borderTop: '1px solid #F5F5F4',
  },
  actionBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '6px 12px',
    backgroundColor: 'transparent',
    border: '1px solid #E7E5E4',
    borderRadius: '7px',
    fontSize: '11px',
    fontWeight: '500',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.1s ease',
  },
  emptyState: {
    gridColumn: '1 / -1',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '64px 20px',
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
  },
  emptyTitle: {
    fontSize: '15px',
    fontWeight: '600',
    color: '#71717A',
    margin: '16px 0 4px',
    fontFamily: "'DM Sans', sans-serif",
  },
  emptyDesc: {
    fontSize: '12px',
    color: '#A1A1AA',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
    textAlign: 'center',
  },
};
