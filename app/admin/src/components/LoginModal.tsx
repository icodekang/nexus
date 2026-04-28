import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';
import {
  type BrowserAccount,
  loginWithBrowserAccount,
  injectBrowserSession,
  initiatePhoneLogin,
  completePhoneLogin,
} from '../api/admin';
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
  exportGuide: string;
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
    exportGuide: 'F12 → Application → Cookies → https://claude.ai → 全选复制',
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
    exportGuide: 'F12 → Application → Cookies → https://chatgpt.com → 全选复制',
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
    exportGuide: 'F12 → Application → Cookies → https://chat.deepseek.com → 全选复制',
  },
};

export default function LoginModal({ account, onClose, onSuccess }: LoginModalProps) {
  const { t } = useI18n();
  const [cookiesJson, setCookiesJson] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [showAutoTry, setShowAutoTry] = useState(false);

  const meta = PROVIDER_META[account.provider] ?? {
    label: account.provider,
    accent: theme.colors.accent.primary,
    icon: null,
    exportGuide: 'F12 → Application → Cookies 面板导出',
  };

  useEffect(() => {
    const timer = setTimeout(() => setMounted(true), 20);
    return () => clearTimeout(timer);
  }, []);

  const handlePasteFromClipboard = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text.trim()) {
        setCookiesJson(text);
        setError(null);
      }
    } catch {
      setError(t('loginModal.clipboardError'));
    }
  };

  const handleInject = async () => {
    if (!cookiesJson.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const parsed = JSON.parse(cookiesJson.trim());
      await injectBrowserSession(account.id, JSON.stringify(parsed));
      setSuccess(true);
      setTimeout(() => onSuccess(), 600);
    } catch (err) {
      if (err instanceof SyntaxError) {
        setError(t('loginModal.jsonError'));
      } else {
        setError(getErrorMessage(err, t));
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ ...styles.overlay, opacity: mounted ? 1 : 0, transition: 'opacity 0.2s ease' }} onClick={onClose}>
      <div
        style={{ ...styles.portal, transform: mounted ? 'translateY(0) scale(1)' : 'translateY(12px) scale(0.98)', transition: 'transform 0.28s cubic-bezier(0.34, 1.56, 0.64, 1)' }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ ...styles.glowStrip, background: meta.accent }} />

        <div style={styles.header}>
          <div style={styles.headerLeft}>
            <div style={{ ...styles.providerIconWrap, borderColor: meta.accent + '30', background: meta.accent + '12' }}>{meta.icon}</div>
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

        {success ? (
          <div style={styles.successState}>
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke={meta.accent} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" style={{ animation: 'fadeIn 0.3s ease' }}>
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            <p style={{ fontSize: '15px', fontWeight: '600', color: theme.colors.text.primary, fontFamily: "'DM Sans', sans-serif", margin: '12px 0 4px' }}>{t('loginModal.successTitle')}</p>
            <p style={{ fontSize: '12px', color: theme.colors.text.tertiary, fontFamily: "'DM Sans', sans-serif", margin: 0 }}>{t('loginModal.successRedirect')}</p>
          </div>
          ) : (
          <div style={styles.form}>
            <div style={styles.guideBox}>
              <div style={{ display: 'flex', gap: '10px', alignItems: 'flex-start' }}>
                <span style={{ ...styles.stepNum, background: meta.accent + '15', color: meta.accent }}>1</span>
                <div>
                  <p style={styles.guideTitle}>{t('loginModal.step1Title')}</p>
                  <p style={styles.guideText}>
                    {t('loginModal.step1Desc', { provider: meta.label })}{' '}
                    <a href="#" onClick={(e) => { e.preventDefault(); setShowAutoTry(v => !v); }} style={{ color: meta.accent, textDecoration: 'none', fontWeight: 600 }}>
                      {showAutoTry ? t('loginModal.autoToggleHide') : t('loginModal.autoToggleShow')}
                    </a>
                  </p>
                </div>
              </div>
              <div style={{ display: 'flex', gap: '10px', alignItems: 'flex-start', marginTop: '14px' }}>
                <span style={{ ...styles.stepNum, background: meta.accent + '15', color: meta.accent }}>2</span>
                <div>
                  <p style={styles.guideTitle}>{t('loginModal.step2Title')}</p>
                  <p style={styles.guideText}>{meta.exportGuide}</p>
                </div>
              </div>
              <div style={{ display: 'flex', gap: '10px', alignItems: 'flex-start', marginTop: '14px' }}>
                <span style={{ ...styles.stepNum, background: meta.accent + '15', color: meta.accent }}>3</span>
                <div>
                  <p style={styles.guideTitle}>{t('loginModal.step3Title')}</p>
                  <p style={styles.guideText}>{t('loginModal.step3Desc')}</p>
                </div>
              </div>
            </div>

            <div style={styles.fieldGroup}>
              <label style={styles.label} htmlFor="inject-cookies">{t('loginModal.cookieJsonLabel')}</label>
              <textarea
                id="inject-cookies"
                value={cookiesJson}
                onChange={(e) => setCookiesJson(e.target.value)}
                style={{ ...styles.textarea, borderColor: loading ? theme.colors.border.default : meta.accent + '40' }}
                rows={8}
                placeholder={`[\n  {"name":"session","value":"xxxx","domain":".${meta.label.toLowerCase().replace('.ai', '.com')}"},\n  ...\n]`}
                disabled={loading}
                spellCheck={false}
              />
            </div>

            <div style={{ display: 'flex', gap: '8px' }}>
              <button type="button" style={{ ...styles.pasteBtn, borderColor: meta.accent + '30', color: meta.accent }} onClick={handlePasteFromClipboard} disabled={loading}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" /></svg>
                {t('loginModal.pasteFromClipboard')}
              </button>
            </div>

            {error && (
              <div style={styles.errorBanner}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <span>{error}</span>
              </div>
            )}

            <button
              type="button"
              disabled={loading || !cookiesJson.trim()}
              style={{ ...styles.submitBtn, background: loading ? theme.colors.text.tertiary : meta.accent, cursor: loading || !cookiesJson.trim() ? 'not-allowed' : 'pointer', opacity: !cookiesJson.trim() ? 0.55 : 1 }}
              onClick={handleInject}
            >
              {loading ? <span style={styles.spinner} /> : t('loginModal.injectBtn')}
            </button>

            <CollapsibleAutoLogin
              open={showAutoTry}
              meta={meta}
              account={account}
              t={t}
              onSuccess={onSuccess}
            />
          </div>
        )}
      </div>

      <style>{`
        @keyframes spin { 100% { transform: rotate(360deg); } }
        @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
        @keyframes slideUp { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
      `}</style>
    </div>
  );
}

function CollapsibleAutoLogin({ open, meta, account, onSuccess, t }: {
  open: boolean;
  meta: typeof PROVIDER_META['claude'];
  account: BrowserAccount;
  onSuccess: () => void;
  t: (key: string, params?: Record<string, string>) => string;
}) {
  const [mode, setMode] = useState<'password' | 'phone'>('password');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [phone, setPhone] = useState('');
  const [code, setCode] = useState('');
  const [phoneStep, setPhoneStep] = useState<'phone' | 'code'>('phone');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [focused, setFocused] = useState<string | null>(null);
  const [countdown, setCountdown] = useState(0);

  useEffect(() => {
    if (countdown <= 0) return;
    const t = setTimeout(() => setCountdown(countdown - 1), 1000);
    return () => clearTimeout(t);
  }, [countdown]);

  if (!open) return null;

  const handlePasswordLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim() || !password.trim()) return;
    setLoading(true); setError(null);
    try { await loginWithBrowserAccount(account.id, email.trim(), password); onSuccess(); }
    catch (err) { setError(getErrorMessage(err, { t: () => '' } as any) || String(err)); }
    finally { setLoading(false); }
  };

  const handleSendCode = async () => {
    if (!phone.trim()) return;
    setLoading(true); setError(null);
    try { await initiatePhoneLogin(account.id, phone.trim()); setPhoneStep('code'); setCountdown(60); }
    catch (err) { setError(getErrorMessage(err, { t: () => '' } as any) || String(err)); }
    finally { setLoading(false); }
  };

  const handleVerifyCode = async () => {
    if (!code.trim()) return;
    setLoading(true); setError(null);
    try { await completePhoneLogin(account.id, code.trim()); onSuccess(); }
    catch (err) { setError(getErrorMessage(err, { t: () => '' } as any) || String(err)); }
    finally { setLoading(false); }
  };

  return (
    <div style={autoStyles.container}>
      <div style={autoStyles.divider}>
        <span style={autoStyles.dividerText}>{t('loginModal.autoDivider')}</span>
      </div>

      <div style={autoStyles.modeToggle}>
        <button style={{ ...autoStyles.modeBtn, color: mode === 'password' ? meta.accent : '#A1A1AA', borderBottomColor: mode === 'password' ? meta.accent : 'transparent' }} onClick={() => setMode('password')}>{t('loginModal.autoPasswordTab')}</button>
        <button style={{ ...autoStyles.modeBtn, color: mode === 'phone' ? meta.accent : '#A1A1AA', borderBottomColor: mode === 'phone' ? meta.accent : 'transparent' }} onClick={() => setMode('phone')}>{t('loginModal.autoPhoneTab')}</button>
      </div>

      {mode === 'password' ? (
        <form onSubmit={handlePasswordLogin} style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          <div style={{ ...styles.inputWrap, borderColor: focused === 'email' ? meta.accent : theme.colors.border.default }}>
            <input id="auto-email" type="email" value={email} onChange={e => setEmail(e.target.value)} onFocus={() => setFocused('email')} onBlur={() => setFocused(null)} style={styles.input} placeholder={t('loginModal.autoEmail')} disabled={loading} autoComplete="username" />
          </div>
          <div style={{ ...styles.inputWrap, borderColor: focused === 'pw' ? meta.accent : theme.colors.border.default }}>
            <input id="auto-pw" type="password" value={password} onChange={e => setPassword(e.target.value)} onFocus={() => setFocused('pw')} onBlur={() => setFocused(null)} style={styles.input} placeholder={t('loginModal.autoPassword')} disabled={loading} autoComplete="current-password" />
          </div>
          {error && <div style={styles.errorBanner}><span>{error.slice(0, 200)}</span></div>}
          <button type="submit" disabled={loading || !email.trim() || !password.trim()} style={{ ...styles.submitBtn, background: meta.accent, opacity: loading ? 0.6 : 1, fontSize: '12px', padding: '10px' }}>
            {loading ? <span style={styles.spinner} /> : t('loginModal.autoLoginBtn')}
          </button>
        </form>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {phoneStep === 'phone' ? (
            <>
              <div style={{ ...styles.inputWrap, borderColor: focused === 'phone' ? meta.accent : theme.colors.border.default }}>
                <input id="auto-phone" type="tel" value={phone} onChange={e => setPhone(e.target.value)} onFocus={() => setFocused('phone')} onBlur={() => setFocused(null)} style={styles.input} placeholder="+86 13800000000" disabled={loading} />
              </div>
              <button type="button" disabled={loading || !phone.trim()} style={{ ...styles.submitBtn, background: meta.accent, opacity: loading ? 0.6 : 1, fontSize: '12px', padding: '10px' }} onClick={handleSendCode}>
                {loading ? <span style={styles.spinner} /> : t('loginModal.autoSendCode')}
              </button>
            </>
          ) : (
            <>
              <div style={styles.codeSentBanner}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={meta.accent} strokeWidth="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" /></svg>
                <span>{t('loginModal.autoSentTo', { phone })}</span>
              </div>
              <div style={{ ...styles.inputWrap, borderColor: focused === 'code' ? meta.accent : theme.colors.border.default }}>
                <input id="auto-code" type="text" value={code} onChange={e => setCode(e.target.value)} onFocus={() => setFocused('code')} onBlur={() => setFocused(null)} style={styles.input} placeholder="000000" disabled={loading} maxLength={6} />
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button type="button" style={{ ...styles.pasteBtn, borderColor: meta.accent + '30', color: meta.accent, flex: 1 }} onClick={() => setPhoneStep('phone')} disabled={loading}>{t('loginModal.autoChangePhone')}</button>
                <button type="button" disabled={loading || !code.trim()} style={{ ...styles.submitBtn, background: meta.accent, opacity: loading ? 0.6 : 1, fontSize: '12px', padding: '10px', flex: 1, marginTop: 0 }} onClick={handleVerifyCode}>
                  {loading ? <span style={styles.spinner} /> : t('loginModal.autoVerify')}
                </button>
              </div>
            </>
          )}
          {error && <div style={styles.errorBanner}><span>{error.slice(0, 200)}</span></div>}
        </div>
      )}
    </div>
  );
}

const autoStyles: Record<string, React.CSSProperties> = {
  container: {
    marginTop: '4px',
  },
  divider: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    padding: '8px 0',
  },
  dividerText: {
    fontSize: '10px',
    color: theme.colors.text.tertiary,
    fontFamily: "'DM Sans', sans-serif",
    whiteSpace: 'nowrap',
  },
  modeToggle: {
    display: 'flex',
    gap: '0',
    marginBottom: '10px',
    borderBottom: `1px solid ${theme.colors.border.subtle}`,
  },
  modeBtn: {
    flex: 1,
    padding: '6px 0',
    border: 'none',
    borderBottom: '2px solid',
    background: 'transparent',
    fontSize: '11px',
    fontFamily: "'DM Sans', sans-serif",
    fontWeight: '600',
    cursor: 'pointer',
    transition: 'all 0.12s ease',
  },
};

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
    maxWidth: '480px',
    boxShadow: '0 24px 64px rgba(0,0,0,0.18), 0 4px 16px rgba(0,0,0,0.08)',
    overflow: 'hidden',
    position: 'relative',
  },
  glowStrip: { height: '3px', width: '100%' },
  header: {
    display: 'flex', justifyContent: 'space-between', alignItems: 'center',
    padding: '20px 24px 14px',
  },
  headerLeft: { display: 'flex', alignItems: 'center', gap: '12px' },
  providerIconWrap: {
    width: '44px', height: '44px', borderRadius: '12px',
    border: '1.5px solid',
    display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0,
  },
  providerLabel: {
    fontSize: '11px', fontWeight: '600', color: theme.colors.text.tertiary,
    fontFamily: "'DM Sans', sans-serif", letterSpacing: '0.04em', textTransform: 'uppercase',
    margin: 0, marginBottom: '2px',
  },
  title: {
    fontSize: '17px', fontWeight: '700', color: theme.colors.text.primary,
    margin: 0, fontFamily: "'Instrument Sans', sans-serif", letterSpacing: '-0.01em',
  },
  closeBtn: {
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    width: '32px', height: '32px', background: 'transparent',
    border: 'none', borderRadius: '8px', cursor: 'pointer',
    color: theme.colors.text.tertiary, flexShrink: 0,
  },
  form: {
    padding: '20px 24px 20px',
    display: 'flex', flexDirection: 'column', gap: '14px',
  },
  successState: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    padding: '48px 24px',
  },
  guideBox: {
    padding: '16px',
    backgroundColor: '#F8F8FB',
    borderRadius: '12px',
    border: '1px solid #EEEEF2',
  },
  stepNum: {
    width: '22px', height: '22px', borderRadius: '50%',
    display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
    fontSize: '12px', fontWeight: '700', fontFamily: "'DM Sans', sans-serif",
    flexShrink: 0, marginTop: '1px',
  },
  guideTitle: {
    margin: 0, fontSize: '13px', fontWeight: '600',
    color: theme.colors.text.primary, fontFamily: "'DM Sans', sans-serif",
  },
  guideText: {
    margin: '2px 0 0', fontSize: '12px', color: theme.colors.text.tertiary,
    fontFamily: "'DM Sans', sans-serif", lineHeight: '1.5',
  },
  fieldGroup: {
    display: 'flex', flexDirection: 'column', gap: '6px',
  },
  label: {
    fontSize: '13px', fontWeight: '600', color: theme.colors.text.primary,
    fontFamily: "'DM Sans', sans-serif", letterSpacing: '-0.01em',
  },
  textarea: {
    width: '100%', padding: '12px 14px', borderRadius: '10px',
    border: '1.5px solid', fontSize: '12px',
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    color: theme.colors.text.primary, backgroundColor: theme.colors.bg.page,
    resize: 'vertical', outline: 'none', lineHeight: '1.5', boxSizing: 'border-box',
  } as React.CSSProperties,
  pasteBtn: {
    padding: '8px 14px', borderRadius: '8px', border: '1.5px solid',
    background: 'transparent', fontSize: '12px', fontWeight: '600',
    fontFamily: "'DM Sans', sans-serif", cursor: 'pointer',
    display: 'flex', alignItems: 'center', gap: '6px', transition: 'all 0.15s ease',
  },
  errorBanner: {
    display: 'flex', alignItems: 'center', gap: '8px',
    padding: '10px 14px', backgroundColor: '#FEF2F2', borderRadius: '8px',
    color: '#EF4444', fontSize: '13px', fontFamily: "'DM Sans', sans-serif",
    border: '1px solid #FECACA', animation: 'slideUp 0.2s ease',
  },
  submitBtn: {
    padding: '13px 20px', borderRadius: '10px', color: '#FFFFFF',
    fontSize: '14px', fontWeight: '700', fontFamily: "'DM Sans', sans-serif",
    border: 'none', display: 'flex', alignItems: 'center', justifyContent: 'center',
    gap: '8px', transition: 'all 0.15s ease', marginTop: '2px', letterSpacing: '-0.01em',
  },
  spinner: {
    width: '18px', height: '18px', border: '2px solid rgba(255,255,255,0.35)',
    borderTopColor: '#FFFFFF', borderRadius: '50%',
    animation: 'spin 0.7s linear infinite', display: 'inline-block',
  },
  inputWrap: {
    display: 'flex', alignItems: 'center', gap: '10px', padding: '0 14px',
    borderRadius: '10px', border: '1.5px solid',
    backgroundColor: theme.colors.bg.page, transition: 'all 0.15s ease',
  },
  input: {
    flex: 1, padding: '10px 0', border: 'none', outline: 'none',
    fontSize: '13px', fontFamily: "'DM Sans', sans-serif",
    color: theme.colors.text.primary, backgroundColor: 'transparent', width: '100%',
  },
  codeSentBanner: {
    display: 'flex', alignItems: 'center', gap: '8px',
    padding: '10px 14px', backgroundColor: '#F0FDF4', borderRadius: '8px',
    border: '1px solid #BBF7D0', fontSize: '12px', fontWeight: '500',
    color: '#166534', fontFamily: "'DM Sans', sans-serif",
  },
};
