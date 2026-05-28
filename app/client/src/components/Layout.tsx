import { useState, useRef, useEffect } from 'react';
import { NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { MessageSquare, Key, BookOpen, Layers, LogOut, Zap, Menu, X, Wallet, ChevronDown, User } from 'lucide-react';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import LoginModal from './LoginModal';
import './Layout.css';

export default function Layout() {
  const { user, logout, isAuthenticated, setShowLoginModal } = useAuthStore();
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const userMenuRef = useRef<HTMLDivElement>(null);
  const { t, locale, setLocale } = useI18n();

  const navItems = [
    { to: '/chat', label: t('layout.chat'), icon: MessageSquare },
    { to: '/models', label: t('layout.models'), icon: Layers },
    { to: '/keys', label: t('layout.apiKeys'), icon: Key },
    { to: '/balance', label: t('layout.balance'), icon: Wallet },
    { to: '/guide', label: t('layout.guide'), icon: BookOpen },
  ];

  useEffect(() => {
    setMobileMenuOpen(false);
  }, [location.pathname]);

  useEffect(() => {
    if (!mobileMenuOpen && !userMenuOpen) return;
    const handler = (e: MouseEvent) => {
      const target = e.target as Node;
      if (menuRef.current && !menuRef.current.contains(target)) setMobileMenuOpen(false);
      if (userMenuRef.current && !userMenuRef.current.contains(target)) setUserMenuOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [mobileMenuOpen, userMenuOpen]);

  return (
    <div className="layout">
      <LoginModal />

      {/* Top Navigation Bar */}
      <header className="topbar">
        <div className="topbar-inner">
          <NavLink to="/" className="topbar-brand">
            <div className="topbar-logo">
              <Zap size={15} strokeWidth={2.5} />
            </div>
            <span className="topbar-brand-text">{t('common.brandName')}</span>
          </NavLink>

          <nav className="topbar-nav">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  `topbar-nav-item ${isActive ? 'active' : ''}`
                }
              >
                <item.icon size={15} strokeWidth={1.75} />
                <span>{item.label}</span>
              </NavLink>
            ))}
          </nav>

          <div className="topbar-right">
            {/* Language toggle */}
            <button
              className="topbar-lang-btn"
              onClick={() => setLocale(locale === 'en' ? 'zh' : 'en')}
              title={t('common.switchLang')}
            >
              {locale === 'en' ? t('lang.toggleZh') : t('lang.toggleEn')}
            </button>

            {/* Mobile hamburger */}
            <button className="topbar-mobile-menu-btn" onClick={() => setMobileMenuOpen(!mobileMenuOpen)}>
              {mobileMenuOpen ? <X size={18} /> : <Menu size={18} />}
            </button>

            {/* User menu / Sign in */}
            <div className="topbar-user-menu" ref={userMenuRef}>
              {isAuthenticated ? (
                <>
                  <button
                    className="topbar-user-btn"
                    onClick={() => setUserMenuOpen(!userMenuOpen)}
                  >
                    <div className="topbar-user-avatar">
                      {user?.email?.charAt(0).toUpperCase() || 'U'}
                    </div>
                    <ChevronDown size={12} />
                  </button>
                  {userMenuOpen && (
                    <div className="topbar-user-dropdown">
                      <div className="topbar-user-dropdown-info">
                        <span className="topbar-user-dropdown-email">{user?.email || t('common.user')}</span>
                      </div>
                      <button
                        className="topbar-user-dropdown-item"
                        onClick={() => { logout(); navigate('/'); setUserMenuOpen(false); }}
                      >
                        <LogOut size={14} />
                        {t('common.signOut')}
                      </button>
                    </div>
                  )}
                </>
              ) : (
                <button
                  className="topbar-signin-btn"
                  onClick={() => setShowLoginModal(true)}
                >
                  <User size={14} />
                  <span>{t('login.signIn')}</span>
                </button>
              )}
            </div>
          </div>
        </div>
      </header>

      {/* Mobile nav dropdown */}
      {mobileMenuOpen && (
        <div className="mobile-nav-dropdown" ref={menuRef}>
          <nav className="mobile-nav-list">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  `mobile-nav-item ${isActive ? 'active' : ''}`
                }
              >
                <item.icon size={18} strokeWidth={1.75} />
                <span>{item.label}</span>
              </NavLink>
            ))}
          </nav>
          <div className="mobile-nav-footer">
            {isAuthenticated ? (
              <button
                className="mobile-nav-logout"
                onClick={() => { logout(); navigate('/'); setMobileMenuOpen(false); }}
              >
                <LogOut size={16} />
                <span>{t('common.signOut')}</span>
              </button>
            ) : (
              <button
                className="mobile-nav-signin"
                onClick={() => { setShowLoginModal(true); setMobileMenuOpen(false); }}
              >
                <User size={16} />
                <span>{t('login.signIn')}</span>
              </button>
            )}
          </div>
        </div>
      )}

      {/* Main content */}
      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
