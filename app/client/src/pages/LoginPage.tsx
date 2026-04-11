import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Zap, Mail, Lock, ArrowRight, AlertCircle } from 'lucide-react';
import { login, register } from '../api/client';
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
    // Skip API call — mock login for preview
    saveAuth('preview-token', { id: '0', email: email || 'preview@nexus.io', subscription_plan: 'free' });
    navigate('/chat');
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
