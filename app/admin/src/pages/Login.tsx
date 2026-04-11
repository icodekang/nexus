import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../auth';
import { useI18n } from '../i18n';

export default function Login() {
  const { login, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const { t, locale, setLocale } = useI18n();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  // Already logged in, redirect
  if (isAuthenticated) {
    navigate('/dashboard', { replace: true });
    return null;
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError('');
    try {
      const success = await login(email, password);
      if (success) {
        navigate('/dashboard', { replace: true });
      }
    } catch {
      setError(t('login.invalidCredentials'));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <div style={styles.header}>
          <div style={styles.logoMark}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
            </svg>
          </div>
          <h1 style={styles.title}>{t('login.title')}</h1>
          <p style={styles.subtitle}>{t('login.subtitle')}</p>
        </div>

        <form onSubmit={handleSubmit} style={styles.form}>
          {error && <div style={styles.error}>{error}</div>}

          <div style={styles.inputGroup}>
            <label style={styles.label}>{t('login.email')}</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t('login.emailPlaceholder')}
              style={styles.input}
              required
            />
          </div>

          <div style={styles.inputGroup}>
            <label style={styles.label}>{t('login.password')}</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={t('login.passwordPlaceholder')}
              style={styles.input}
              required
            />
          </div>

          <button
            type="submit"
            style={{
              ...styles.submitBtn,
              opacity: isLoading ? 0.7 : 1,
            }}
            disabled={isLoading}
          >
            {isLoading ? t('login.signingIn') : t('login.signIn')}
          </button>
        </form>

        <div style={styles.footerRow}>
          <p style={styles.footer}>{t('login.footer')}</p>
          <button
            onClick={() => setLocale(locale === 'en' ? 'zh' : 'en')}
            style={styles.langBtn}
            title={locale === 'en' ? '切换到中文' : 'Switch to English'}
          >
            {locale === 'en' ? '中文' : 'EN'}
          </button>
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: '100vh',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FAFAF9',
    padding: '24px',
  },
  card: {
    width: '100%',
    maxWidth: '380px',
    backgroundColor: '#FFFFFF',
    borderRadius: '16px',
    padding: '40px 36px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04), 0 8px 24px rgba(0,0,0,0.06)',
  },
  header: {
    textAlign: 'center',
    marginBottom: '36px',
  },
  logoMark: {
    width: '44px',
    height: '44px',
    backgroundColor: '#6366F1',
    borderRadius: '12px',
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: '20px',
  },
  title: {
    fontSize: '22px',
    fontWeight: '700',
    color: '#18181B',
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.02em',
  },
  subtitle: {
    fontSize: '13px',
    color: '#71717A',
    marginTop: '6px',
    fontFamily: "'DM Sans', sans-serif",
  },
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: '18px',
  },
  error: {
    padding: '10px 14px',
    backgroundColor: 'rgba(239, 68, 68, 0.06)',
    border: '1px solid rgba(239, 68, 68, 0.15)',
    borderRadius: '10px',
    fontSize: '13px',
    color: '#EF4444',
    fontFamily: "'DM Sans', sans-serif",
  },
  inputGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
  },
  label: {
    fontSize: '12px',
    fontWeight: '500',
    color: '#52525B',
    fontFamily: "'DM Sans', sans-serif",
  },
  input: {
    padding: '11px 14px',
    backgroundColor: '#F5F5F4',
    border: '1px solid transparent',
    borderRadius: '10px',
    fontSize: '14px',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
    outline: 'none',
    transition: 'all 0.15s ease',
  },
  submitBtn: {
    padding: '12px',
    backgroundColor: '#6366F1',
    border: 'none',
    borderRadius: '10px',
    fontSize: '14px',
    fontWeight: '600',
    color: '#FFFFFF',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.15s ease',
    marginTop: '4px',
  },
  footerRow: {
    marginTop: '28px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '12px',
  },
  footer: {
    textAlign: 'center',
    fontSize: '11px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
    margin: 0,
  },
  langBtn: {
    padding: '3px 8px',
    fontSize: '11px',
    fontWeight: '500',
    color: '#71717A',
    backgroundColor: 'transparent',
    border: '1px solid #E7E5E4',
    borderRadius: '6px',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.15s ease',
  },
};
