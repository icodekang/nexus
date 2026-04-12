import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Zap, Mail, Lock, ArrowRight, AlertCircle } from 'lucide-react';
import { login, register, ApiError } from '../api/client';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import './LoginPage.css';

export default function LoginPage() {
  const navigate = useNavigate();
  const { login: saveAuth } = useAuthStore();
  const { t } = useI18n();
  const [isRegister, setIsRegister] = useState(false);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      const res = isRegister
        ? await register(email, password)
        : await login(email, password);
      saveAuth(res.token, res.user);
      navigate('/chat');
    } catch (err: unknown) {
      if (err instanceof ApiError) {
        switch (err.code) {
          case 'invalid_credentials': setError(t('common.invalidCredentials')); break;
          case 'user_already_exists': setError(t('common.userAlreadyExists')); break;
          case 'network_error': setError(t('common.networkError')); break;
          case 'internal_error': setError(t('common.serverError')); break;
          default: setError(err.message || t('login.authFailed'));
        }
      } else {
        setError(t('login.authFailed'));
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="login-brand">
          <div className="login-logo">
            <Zap size={22} strokeWidth={2.5} />
          </div>
          <h1 className="login-brand-name">{t('common.brandName')}</h1>
          <p className="login-brand-tagline">{t('login.aiGateway')}</p>
        </div>

        <form className="login-form" onSubmit={handleSubmit}>
          <h2 className="login-heading">{isRegister ? t('login.createAccount') : t('login.signIn')}</h2>

          {error && (
            <div className="login-error">
              <AlertCircle size={14} />
              {error}
            </div>
          )}

          <div className="login-field">
            <Mail size={16} className="login-field-icon" />
            <input
              type="email"
              placeholder={t('login.emailAddress')}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="login-input"
              required
              autoFocus
            />
          </div>

          <div className="login-field">
            <Lock size={16} className="login-field-icon" />
            <input
              type="password"
              placeholder={t('login.password')}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="login-input"
              required
              minLength={6}
            />
          </div>

          <button className="login-submit" type="submit" disabled={loading}>
            {loading ? t('login.pleaseWait') : isRegister ? t('login.createAccount') : t('login.signIn')}
            <ArrowRight size={16} />
          </button>
        </form>

        <div className="login-switch">
          {isRegister ? t('login.alreadyHaveAccount') : t('login.noAccount')}
          <button className="login-switch-btn" onClick={() => { setIsRegister(!isRegister); setError(''); }}>
            {isRegister ? t('login.signInLink') : t('login.createOneLink')}
          </button>
        </div>
      </div>
    </div>
  );
}
