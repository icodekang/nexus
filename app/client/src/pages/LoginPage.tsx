import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { Zap, Mail, Phone, Lock, ArrowRight, AlertCircle, ShieldCheck, ArrowLeft } from 'lucide-react';
import { login, register, sendSmsCode, verifySmsCode } from '../api/client';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import { getErrorMessage } from '../utils/errors';
import './LoginPage.css';

const COUNTDOWN_SECONDS = 60;

type AuthMethod = 'email' | 'phone';
type EmailStep = 'login' | 'register';

export default function LoginPage() {
  const navigate = useNavigate();
  const { login: storeLogin } = useAuthStore();
  const { t } = useI18n();

  // Auth method selection
  const [authMethod, setAuthMethod] = useState<AuthMethod>('email');

  // Email state
  const [emailStep, setEmailStep] = useState<EmailStep>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [emailLoading, setEmailLoading] = useState(false);

  // Phone state
  const [step, setStep] = useState<'phone' | 'code'>('phone');
  const [phone, setPhone] = useState('');
  const [countryCode, setCountryCode] = useState('+86');
  const [code, setCode] = useState('');
  const [sendingCode, setSendingCode] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const [codeSent, setCodeSent] = useState(false);

  // Shared state
  const [error, setError] = useState('');
  const codeInputRef = useRef<HTMLInputElement>(null);
  const emailInputRef = useRef<HTMLInputElement>(null);
  const phoneInputRef = useRef<HTMLInputElement>(null);

  // Countdown timer for phone
  useEffect(() => {
    if (countdown <= 0) return;
    const timer = setInterval(() => {
      setCountdown((c) => c - 1);
    }, 1000);
    return () => clearInterval(timer);
  }, [countdown]);

  const fullPhone = countryCode + phone.replace(/\D/g, '');

  // Email login/register
  const handleEmailSubmit = async (e: React.FormEvent) => {
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
        navigate('/chat');
      } else {
        const res = await register(email, password);
        storeLogin(res.token, res.user);
        navigate('/chat');
      }
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setEmailLoading(false);
    }
  };

  // Phone: send SMS code
  const handleSendCode = async (e: React.FormEvent) => {
    e.preventDefault();
    if (phone.replace(/\D/g, '').length < 10) {
      setError(t('login.invalidPhone'));
      return;
    }
    setSendingCode(true);
    setError('');
    try {
      await sendSmsCode(fullPhone);
      setCodeSent(true);
      setCountdown(COUNTDOWN_SECONDS);
      setStep('code');
      setTimeout(() => codeInputRef.current?.focus(), 100);
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setSendingCode(false);
    }
  };

  // Phone: verify code
  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault();
    if (code.replace(/\D/g, '').length < 4) {
      setError(t('login.invalidCode'));
      return;
    }
    setEmailLoading(true); // reuse loading state
    setError('');
    try {
      const res = await verifySmsCode(fullPhone, code);
      storeLogin(res.token, res.user);
      navigate('/chat');
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setEmailLoading(false);
    }
  };

  // Phone: resend code
  const handleResend = async () => {
    if (countdown > 0) return;
    setSendingCode(true);
    setError('');
    try {
      await sendSmsCode(fullPhone);
      setCountdown(COUNTDOWN_SECONDS);
    } catch (err) {
      setError(getErrorMessage(err, t));
    } finally {
      setSendingCode(false);
    }
  };

  // Switch auth method
  const handleSwitchMethod = (method: AuthMethod) => {
    setAuthMethod(method);
    setError('');
    setStep('phone');
    setCode('');
    setCodeSent(false);
    setEmail('');
    setPassword('');
    setConfirmPassword('');
    setEmailStep('login');
  };

  // Switch between login and register (email)
  const handleSwitchEmailStep = () => {
    setEmailStep(emailStep === 'login' ? 'register' : 'login');
    setError('');
    setPassword('');
    setConfirmPassword('');
  };

  // Back to phone input from code entry
  const handleBackToPhone = () => {
    setStep('phone');
    setCode('');
    setError('');
    setCodeSent(false);
    setTimeout(() => phoneInputRef.current?.focus(), 100);
  };

  return (
    <div className="login-page">
      <div className="login-card">
        <div className="login-brand">
          <div className="login-logo">
            <Zap size={22} strokeWidth={2.5} />
          </div>
          <h1 className="login-brand-name">{t('common.brandName')}</h1>
        </div>

        {/* Auth method tabs */}
        <div className="login-auth-tabs">
          <button
            className={`login-auth-tab ${authMethod === 'email' ? 'active' : ''}`}
            onClick={() => handleSwitchMethod('email')}
          >
            <Mail size={16} />
            <span>{t('login.emailTab')}</span>
          </button>
          <button
            className={`login-auth-tab ${authMethod === 'phone' ? 'active' : ''}`}
            onClick={() => handleSwitchMethod('phone')}
          >
            <Phone size={16} />
            <span>{t('login.phoneTab')}</span>
          </button>
        </div>

        {error && (
          <div className="login-error">
            <AlertCircle size={14} />
            {error}
          </div>
        )}

        {/* Email Auth Form */}
        {authMethod === 'email' && (
          <form className="login-form" onSubmit={handleEmailSubmit}>
            <h2 className="login-heading">
              {emailStep === 'login' ? t('login.signIn') : t('login.createAccount')}
            </h2>

            <div className="login-field-row">
              <Mail size={16} className="login-field-icon" />
              <input
                ref={emailInputRef}
                type="email"
                placeholder={t('login.emailPlaceholder')}
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="login-input login-field-input"
                required
                autoFocus
              />
            </div>

            <div className="login-field-row">
              <Lock size={16} className="login-field-icon" />
              <input
                type="password"
                placeholder={t('login.password')}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="login-input login-field-input"
                required
              />
            </div>

            {emailStep === 'register' && (
              <div className="login-field-row">
                <Lock size={16} className="login-field-icon" />
                <input
                  type="password"
                  placeholder={t('login.confirmPassword')}
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="login-input login-field-input"
                  required
                />
              </div>
            )}

            <button className="login-submit" type="submit" disabled={emailLoading}>
              {emailLoading ? t('login.pleaseWait') : (emailStep === 'login' ? t('login.signIn') : t('login.createAccount'))}
              {!emailLoading && <ArrowRight size={16} />}
            </button>

            <div className="login-switch-row">
              <span className="login-switch-text">
                {emailStep === 'login' ? t('login.noAccount') : t('login.alreadyHaveAccount')}
              </span>
              <button
                type="button"
                className="login-switch-btn"
                onClick={handleSwitchEmailStep}
              >
                {emailStep === 'login' ? t('login.createOneLink') : t('login.signInLink')}
              </button>
            </div>
          </form>
        )}

        {/* Phone Auth Form */}
        {authMethod === 'phone' && step === 'phone' && (
          <form className="login-form" onSubmit={handleSendCode}>
            <h2 className="login-heading">{t('login.enterPhone')}</h2>
            <p className="login-subheading">{t('login.phoneHint')}</p>

            <div className="login-phone-row">
              <select
                className="login-country-select"
                value={countryCode}
                onChange={(e) => setCountryCode(e.target.value)}
              >
                <option value="+86">+86</option>
                <option value="+1">+1</option>
                <option value="+44">+44</option>
                <option value="+81">+81</option>
                <option value="+82">+82</option>
                <option value="+65">+65</option>
                <option value="+91">+91</option>
                <option value="+62">+62</option>
                <option value="+63">+63</option>
                <option value="+84">+84</option>
                <option value="+60">+60</option>
                <option value="+66">+66</option>
                <option value="+886">+886</option>
                <option value="+852">+852</option>
                <option value="+853">+853</option>
              </select>
              <input
                ref={phoneInputRef}
                type="tel"
                placeholder={t('login.phonePlaceholder')}
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                className="login-input login-phone-input"
                required
                autoFocus
              />
            </div>

            <button className="login-submit" type="submit" disabled={sendingCode}>
              {sendingCode ? t('login.sending') : t('login.sendCode')}
              {!sendingCode && <ArrowRight size={16} />}
            </button>
          </form>
        )}

        {/* Phone: SMS Code Verification */}
        {authMethod === 'phone' && step === 'code' && (
          <form className="login-form" onSubmit={handleVerify}>
            <h2 className="login-heading">{t('login.enterCode')}</h2>
            <p className="login-subheading">
              {t('login.codeSentTo')}
              <span className="login-phone-highlight">{fullPhone}</span>
            </p>

            <div className="login-code-row">
              <ShieldCheck size={18} className="login-field-icon login-code-icon" />
              <input
                ref={codeInputRef}
                type="text"
                inputMode="numeric"
                pattern="[0-9]*"
                placeholder={t('login.codePlaceholder')}
                value={code}
                onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                className="login-input login-code-input"
                required
                autoFocus
              />
            </div>

            <button
              className="login-submit"
              type="submit"
              disabled={emailLoading || code.length < 4}
            >
              {emailLoading ? t('login.verifying') : t('login.verify')}
              {!emailLoading && <ArrowRight size={16} />}
            </button>

            <div className="login-resend-row">
              <span className="login-resend-text">{t('login.noCodeReceived')}</span>
              <button
                type="button"
                className="login-resend-btn"
                onClick={handleResend}
                disabled={countdown > 0 || sendingCode}
              >
                {countdown > 0
                  ? t('login.resendIn', { seconds: countdown })
                  : sendingCode
                  ? t('login.sending')
                  : t('login.resend')}
              </button>
            </div>

            <button type="button" className="login-back-btn" onClick={handleBackToPhone}>
              <ArrowLeft size={14} />
              {t('login.changePhone')}
            </button>
          </form>
        )}
      </div>
    </div>
  );
}