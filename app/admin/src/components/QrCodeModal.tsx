/**
 * @file LoginModal - 账号密码登录弹窗组件
 * 使用无头浏览器填表方式登录，登录成功后自动关闭弹窗
 */
import { useState } from 'react';
import { useI18n } from '../i18n';
import { type BrowserAccount, loginWithBrowserAccount } from '../api/admin';

interface LoginModalProps {
  account: BrowserAccount;
  onClose: () => void;
  onSuccess: () => void;
}

export default function LoginModal({ account, onClose, onSuccess }: LoginModalProps) {
  const { t } = useI18n();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email || !password) {
      setError('Please enter email and password');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await loginWithBrowserAccount(account.id, email, password);
      onSuccess();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div style={styles.header}>
          <h2 style={styles.title}>{t('loginModal.title') || '账号密码登录'}</h2>
          <button style={styles.closeBtn} onClick={onClose}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} style={styles.form}>
          {error && (
            <div style={styles.error}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <span>{error}</span>
            </div>
          )}

          <div style={styles.providerBadge}>
            {account.provider === 'claude' && <span style={{ ...styles.badgeText, color: '#D97706' }}>Claude.ai</span>}
            {account.provider === 'chatgpt' && <span style={{ ...styles.badgeText, color: '#10A37F' }}>ChatGPT</span>}
            {account.provider === 'deepseek' && <span style={{ ...styles.badgeText, color: '#0068FF' }}>DeepSeek</span>}
          </div>

          <div style={styles.field}>
            <label style={styles.label}>{t('loginModal.email') || '邮箱'}</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              style={styles.input}
              placeholder="email@example.com"
              disabled={loading}
              autoFocus
            />
          </div>

          <div style={styles.field}>
            <label style={styles.label}>{t('loginModal.password') || '密码'}</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              style={styles.input}
              placeholder="••••••••"
              disabled={loading}
            />
          </div>

          <button type="submit" style={loading ? styles.buttonDisabled : styles.button} disabled={loading}>
            {loading ? (
              <span style={styles.spinner} />
            ) : (
              t('loginModal.loginBtn') || '登录'
            )}
          </button>
        </form>

        <p style={styles.hint}>
          {t('loginModal.hint') || '无头浏览器将自动填写表单并登录'}
        </p>
      </div>

      <style>{`
        @keyframes spin {
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0, left: 0, right: 0, bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    backgroundColor: '#FFFFFF',
    borderRadius: '16px',
    width: '100%',
    maxWidth: '400px',
    boxShadow: '0 20px 40px rgba(0, 0, 0, 0.15)',
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '20px 24px',
    borderBottom: '1px solid #F5F5F4',
  },
  title: {
    fontSize: '16px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
  },
  closeBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '32px',
    height: '32px',
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '8px',
    cursor: 'pointer',
    color: '#71717A',
  },
  form: {
    padding: '24px',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  error: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '12px',
    backgroundColor: '#FEF2F2',
    borderRadius: '8px',
    color: '#EF4444',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
  },
  providerBadge: {
    display: 'flex',
    justifyContent: 'center',
    padding: '4px 12px',
    borderRadius: '9999px',
    backgroundColor: '#F5F5F4',
  },
  badgeText: {
    fontSize: '12px',
    fontWeight: '600',
    fontFamily: "'DM Sans', sans-serif",
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  label: {
    fontSize: '13px',
    fontWeight: '500',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  input: {
    padding: '10px 12px',
    borderRadius: '8px',
    border: '1px solid #E4E4E7',
    fontSize: '14px',
    fontFamily: "'DM Sans', sans-serif",
    outline: 'none',
    transition: 'border-color 0.2s',
    backgroundColor: '#FFFFFF',
  },
  button: {
    padding: '12px',
    borderRadius: '8px',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    fontSize: '14px',
    fontWeight: '600',
    fontFamily: "'DM Sans', sans-serif",
    border: 'none',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  buttonDisabled: {
    padding: '12px',
    borderRadius: '8px',
    backgroundColor: '#A5A5AF',
    color: '#FFFFFF',
    fontSize: '14px',
    fontWeight: '600',
    fontFamily: "'DM Sans', sans-serif",
    border: 'none',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  spinner: {
    width: '20px',
    height: '20px',
    border: '2px solid rgba(255,255,255,0.3)',
    borderTopColor: '#FFFFFF',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  hint: {
    padding: '0 24px 20px',
    fontSize: '11px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
    textAlign: 'center',
    margin: 0,
  },
};