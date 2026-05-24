import { useState, useRef } from 'react';
import { Mail, Lock, ArrowRight, AlertCircle, X } from 'lucide-react';
import { login, register } from '../api/client';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import { getErrorMessage } from '../utils/errors';

export default function LoginModal() {
  const { showLoginModal, setShowLoginModal, login: storeLogin } = useAuthStore();
  const { t } = useI18n();

  const [emailStep, setEmailStep] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [emailLoading, setEmailLoading] = useState(false);
  const [error, setError] = useState('');
  const emailInputRef = useRef<HTMLInputElement>(null);

  if (!showLoginModal) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!email.trim() || !password.trim()) {
      setError(t('login.fillAllFields'));
      return;
    }

    if (emailStep === 'register' && password !== confirmPassword) {
      setError(t('login.passwordMismatch'));
      return;
    }

    if (password.length < 6) {
      setError(t('login.passwordTooShort'));
      return;
    }

    setEmailLoading(true);
    try {
      if (emailStep === 'login') {
        const res = await login(email, password);
        storeLogin(res.token, res.user);
      } else {
        const res = await register(email, password);
        storeLogin(res.token, res.user);
      }
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setEmailLoading(false);
    }
  };

  const handleSwitchStep = () => {
    setEmailStep(emailStep === 'login' ? 'register' : 'login');
    setError('');
    setPassword('');
    setConfirmPassword('');
  };

  const handleClose = () => {
    setShowLoginModal(false);
    setError('');
    setEmail('');
    setPassword('');
    setConfirmPassword('');
    setEmailStep('login');
  };

  return (
    <div className="login-modal-overlay" onClick={handleClose}>
      <div className="login-modal-card" onClick={(e) => e.stopPropagation()}>
        <button className="login-modal-close" onClick={handleClose}>
          <X size={18} />
        </button>

        <div className="login-modal-brand">
          <div className="login-modal-logo">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
            </svg>
          </div>
          <span className="login-modal-brand-text">{t('common.brandName')}</span>
        </div>

        <h2 className="login-modal-heading">
          {emailStep === 'login' ? t('login.signIn') : t('login.createAccount')}
        </h2>

        {error && (
          <div className="login-modal-error">
            <AlertCircle size={14} />
            {error}
          </div>
        )}

        <form className="login-modal-form" onSubmit={handleSubmit}>
          <div className="login-modal-field">
            <Mail size={16} className="login-modal-field-icon" />
            <input
              ref={emailInputRef}
              type="email"
              placeholder={t('login.emailPlaceholder')}
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="login-modal-input"
              required
              autoFocus
            />
          </div>

          <div className="login-modal-field">
            <Lock size={16} className="login-modal-field-icon" />
            <input
              type="password"
              placeholder={t('login.password')}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="login-modal-input"
              required
            />
          </div>

          {emailStep === 'register' && (
            <div className="login-modal-field">
              <Lock size={16} className="login-modal-field-icon" />
              <input
                type="password"
                placeholder={t('login.confirmPassword')}
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="login-modal-input"
                required
              />
            </div>
          )}

          <button className="login-modal-submit" type="submit" disabled={emailLoading}>
            {emailLoading ? t('login.pleaseWait') : (emailStep === 'login' ? t('login.signIn') : t('login.createAccount'))}
            {!emailLoading && <ArrowRight size={16} />}
          </button>

          <div className="login-modal-switch">
            <span className="login-modal-switch-text">
              {emailStep === 'login' ? t('login.noAccount') : t('login.alreadyHaveAccount')}
            </span>
            <button type="button" className="login-modal-switch-btn" onClick={handleSwitchStep}>
              {emailStep === 'login' ? t('login.createOneLink') : t('login.signInLink')}
            </button>
          </div>
        </form>

        <div className="login-modal-footer">
          <a href="/login" className="login-modal-full-link">
            {t('login.phoneTab')}
          </a>
        </div>
      </div>
    </div>
  );
}
