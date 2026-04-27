/**
 * @file LoginModal - Password-based login for browser accounts
 *
 * Design: "Digital Portal" aesthetic — a focused, dark-framed terminal-like
 * experience that evokes the headless browser underneath. Minimalist but
 * with strong personality: a deep charcoal portal frame, soft glow effects,
 * and smooth spring animations.
 */
import { useState, useRef, useEffect } from 'react';
import { useI18n } from '../i18n';
import { type BrowserAccount, loginWithBrowserAccount } from '../api/admin';
import { getErrorMessage } from '../utils/errors';
import { theme } from '../theme';

interface LoginModalProps {
  account: BrowserAccount;
  onClose: () => void;
  onSuccess: () => void;
}

const PROVIDER_META: Record<string, {
  label: string;
  accent: string;
  icon: React.ReactNode;
}> = {
  claude: {
    label: 'Claude.ai',
    accent: '#D97706',
    icon: (
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" fill="#D97706" opacity="0.15" />
        <path d="M12 6C8.686 6 6 8.686 6 12s2.686 6 6 6 6-2.686 6-6-2.686-6-6-6zm0 10a4 4 0 1 1 0-8 4 4 0 0 1 0 8z" fill="#D97706" />
      </svg>
    ),
  },
  chatgpt: {
    label: 'ChatGPT',
    accent: '#10A37F',
    icon: (
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" fill="#10A37F" opacity="0.15" />
        <path d="M15.5 9.5a2.5 2.5 0 1 1-3.5 3.2 2.5 2.5 0 0 1 3.5-3.2z" fill="#10A37F" />
        <circle cx="9" cy="13" r="1.5" fill="#10A37F" />
      </svg>
    ),
  },
  deepseek: {
    label: 'DeepSeek',
    accent: '#0068FF',
    icon: (
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" fill="#0068FF" opacity="0.15" />
        <path d="M8 10l4 2-4 2V10zm4 2l4-2v4l-4-2z" fill="#0068FF" />
      </svg>
    ),
  },
};

export default function LoginModal({ account, onClose, onSuccess }: LoginModalProps) {
  const { t } = useI18n();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [focusedField, setFocusedField] = useState<string | null>(null);
  const [mounted, setMounted] = useState(false);
  const passwordRef = useRef<HTMLInputElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  const meta = PROVIDER_META[account.provider] ?? {
    label: account.provider,
    accent: theme.colors.accent.primary,
    icon: null,
  };

  useEffect(() => {
    const t = setTimeout(() => setMounted(true), 20);
    return () => clearTimeout(t);
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim() || !password.trim()) {
      setError(t('errors.invalid_credentials'));
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await loginWithBrowserAccount(account.id, email.trim(), password);
      onSuccess();
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') onClose();
  };

  return (
    <div
      ref={overlayRef}
      style={{
        ...styles.overlay,
        opacity: mounted ? 1 : 0,
        transition: 'opacity 0.2s ease',
      }}
      onClick={onClose}
      onKeyDown={handleKeyDown}
    >
      {/* Portal frame */}
      <div
        style={{
          ...styles.portal,
          transform: mounted ? 'translateY(0) scale(1)' : 'translateY(16px) scale(0.97)',
          transition: 'transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Glow strip at top */}
        <div style={{ ...styles.glowStrip, background: meta.accent }} />

        {/* Header */}
        <div style={styles.header}>
          <div style={styles.headerLeft}>
            <div style={{ ...styles.providerIconWrap, borderColor: meta.accent + '30', background: meta.accent + '12' }}>
              {meta.icon}
            </div>
            <div>
              <p style={styles.providerLabel}>{meta.label}</p>
              <h2 style={styles.title}>{t('loginModal.title')}</h2>
            </div>
          </div>
          <button style={styles.closeBtn} onClick={onClose} aria-label="Close">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} style={styles.form} noValidate>
          {/* Email field */}
          <div style={styles.fieldGroup}>
            <label style={styles.label} htmlFor="login-email">
              {t('loginModal.email')}
            </label>
            <div style={{
              ...styles.inputWrap,
              borderColor: focusedField === 'email' ? meta.accent : theme.colors.border.default,
              boxShadow: focusedField === 'email' ? `0 0 0 3px ${meta.accent}18` : 'none',
            }}>
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke={focusedField === 'email' ? meta.accent : theme.colors.text.tertiary} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={styles.fieldIcon}>
                <rect x="2" y="4" width="20" height="16" rx="2" /><path d="M22 6l-10 7L2 6" />
              </svg>
              <input
                id="login-email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                onFocus={() => setFocusedField('email')}
                onBlur={() => setFocusedField(null)}
                style={styles.input}
                placeholder="email@example.com"
                disabled={loading}
                autoFocus
                autoComplete="username"
              />
            </div>
          </div>

          {/* Password field */}
          <div style={styles.fieldGroup}>
            <label style={styles.label} htmlFor="login-password">
              {t('loginModal.password')}
            </label>
            <div style={{
              ...styles.inputWrap,
              borderColor: focusedField === 'password' ? meta.accent : theme.colors.border.default,
              boxShadow: focusedField === 'password' ? `0 0 0 3px ${meta.accent}18` : 'none',
            }}>
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke={focusedField === 'password' ? meta.accent : theme.colors.text.tertiary} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={styles.fieldIcon}>
                <rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" />
              </svg>
              <input
                id="login-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                onFocus={() => setFocusedField('password')}
                onBlur={() => setFocusedField(null)}
                ref={passwordRef}
                style={styles.input}
                placeholder="••••••••"
                disabled={loading}
                autoComplete="current-password"
              />
            </div>
          </div>

          {/* Error message */}
          {error && (
            <div style={styles.errorBanner}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <span>{error}</span>
            </div>
          )}

          {/* Submit */}
          <button
            type="submit"
            disabled={loading || !email.trim() || !password.trim()}
            style={{
              ...styles.submitBtn,
              background: loading ? theme.colors.text.tertiary : meta.accent,
              cursor: loading || !email.trim() || !password.trim() ? 'not-allowed' : 'pointer',
              opacity: !email.trim() || !password.trim() ? 0.6 : 1,
            }}
          >
            {loading ? (
              <span style={styles.spinner} />
            ) : (
              <>
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" /><polyline points="10 17 15 12 10 7" /><line x1="15" y1="12" x2="3" y2="12" />
                </svg>
                {t('loginModal.loginBtn')}
              </>
            )}
          </button>
        </form>

        {/* Footer hint */}
        <p style={styles.hint}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke={theme.colors.text.tertiary} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10" /><path d="M12 16v-4M12 8h.01" />
          </svg>
          {t('loginModal.hint')}
        </p>
      </div>

      <style>{`
        @keyframes spin { 100% { transform: rotate(360deg); } }
        @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
        @keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
      `}</style>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0, left: 0, right: 0, bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.55)',
    backdropFilter: 'blur(6px)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
    padding: '16px',
  },
  portal: {
    backgroundColor: '#FFFFFF',
    borderRadius: '18px',
    width: '100%',
    maxWidth: '400px',
    boxShadow: '0 24px 64px rgba(0,0,0,0.18), 0 4px 16px rgba(0,0,0,0.08)',
    overflow: 'hidden',
    position: 'relative',
  },
  glowStrip: {
    height: '3px',
    width: '100%',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '24px 24px 20px',
    borderBottom: `1px solid ${theme.colors.border.subtle}`,
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  providerIconWrap: {
    width: '44px',
    height: '44px',
    borderRadius: '12px',
    border: '1.5px solid',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
  },
  providerLabel: {
    fontSize: '11px',
    fontWeight: '600',
    color: theme.colors.text.tertiary,
    fontFamily: "'DM Sans', sans-serif",
    letterSpacing: '0.04em',
    textTransform: 'uppercase',
    margin: 0,
    marginBottom: '2px',
  },
  title: {
    fontSize: '17px',
    fontWeight: '700',
    color: theme.colors.text.primary,
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.01em',
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
    color: theme.colors.text.tertiary,
    transition: 'all 0.15s ease',
    flexShrink: 0,
  },
  form: {
    padding: '24px 24px 16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  fieldGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  label: {
    fontSize: '13px',
    fontWeight: '600',
    color: theme.colors.text.primary,
    fontFamily: "'DM Sans', sans-serif",
    letterSpacing: '-0.01em',
  },
  inputWrap: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    padding: '0 14px',
    borderRadius: '10px',
    border: '1.5px solid',
    backgroundColor: theme.colors.bg.page,
    transition: 'all 0.15s ease',
  },
  fieldIcon: {
    flexShrink: 0,
  },
  input: {
    flex: 1,
    padding: '12px 0',
    border: 'none',
    outline: 'none',
    fontSize: '14px',
    fontFamily: "'DM Sans', sans-serif",
    color: theme.colors.text.primary,
    backgroundColor: 'transparent',
    width: '100%',
  },
  errorBanner: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '10px 14px',
    backgroundColor: '#FEF2F2',
    borderRadius: '8px',
    color: '#EF4444',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
    border: '1px solid #FECACA',
    animation: 'slideUp 0.2s ease',
  },
  submitBtn: {
    padding: '13px 20px',
    borderRadius: '10px',
    color: '#FFFFFF',
    fontSize: '14px',
    fontWeight: '700',
    fontFamily: "'DM Sans', sans-serif",
    border: 'none',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '8px',
    transition: 'all 0.15s ease',
    marginTop: '4px',
    letterSpacing: '-0.01em',
  },
  spinner: {
    width: '18px',
    height: '18px',
    border: '2px solid rgba(255,255,255,0.35)',
    borderTopColor: '#FFFFFF',
    borderRadius: '50%',
    animation: 'spin 0.7s linear infinite',
    display: 'inline-block',
  },
  hint: {
    margin: '0 24px 20px',
    padding: '10px 14px',
    backgroundColor: theme.colors.bg.page,
    borderRadius: '8px',
    fontSize: '12px',
    color: theme.colors.text.tertiary,
    fontFamily: "'DM Sans', sans-serif",
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    lineHeight: '1.5',
  },
};