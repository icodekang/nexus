import { useState, useRef, useEffect } from 'react';
import { NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { MessageSquare, Key, BookOpen, Layers, LogOut, Zap, Menu, X, Plus, Trash2, CreditCard } from 'lucide-react';
import { useAuthStore } from '../stores/authStore';
import { useChatStore } from '../stores/chatStore';
import { useI18n } from '../i18n';
import './Layout.css';

export default function Layout() {
  const { user, logout } = useAuthStore();
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);
  const { t, locale, setLocale } = useI18n();
  const {
    selectedModel, createConversation,
    conversations, activeConversationId,
    setActiveConversation, deleteConversation,
  } = useChatStore();

  const navItems = [
    { to: '/chat', label: t('layout.chat'), icon: MessageSquare },
    { to: '/models', label: t('layout.models'), icon: Layers },
    { to: '/keys', label: t('layout.apiKeys'), icon: Key },
    { to: '/subscription', label: t('layout.subscription'), icon: CreditCard },
    { to: '/guide', label: t('layout.guide'), icon: BookOpen },
  ];

  // Close mobile menu on route change
  useEffect(() => {
    setMobileMenuOpen(false);
  }, [location.pathname]);

  // Close mobile menu on outside click
  useEffect(() => {
    if (!mobileMenuOpen) return;
    const handler = (e: MouseEvent) => {
      if (sidebarRef.current && !sidebarRef.current.contains(e.target as Node)) {
        setMobileMenuOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [mobileMenuOpen]);

  return (
    <div className="layout">
      {/* Mobile overlay */}
      {mobileMenuOpen && (
        <div className="mobile-sidebar-overlay" onClick={() => setMobileMenuOpen(false)} />
      )}

      {/* Sidebar — works on both desktop and mobile */}
      <aside ref={sidebarRef} className={`sidebar ${mobileMenuOpen ? 'mobile-open' : ''}`}>
        <div className="sidebar-brand">
          <div className="sidebar-logo">
            <Zap size={18} strokeWidth={2.5} />
          </div>
          <span className="sidebar-brand-text">{t('common.brandName')}</span>
          <button className="sidebar-close-btn" onClick={() => setMobileMenuOpen(false)}>
            <X size={18} />
          </button>
        </div>

        <nav className="sidebar-nav">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `sidebar-nav-item ${isActive ? 'active' : ''}`
              }
            >
              <item.icon size={18} strokeWidth={1.75} />
              <span>{item.label}</span>
            </NavLink>
          ))}

          {/* Mobile-only: inline conversation list */}
          <div className="sidebar-conversations">
            <div className="sidebar-conversations-divider">
              <span className="sidebar-conversations-divider-label">{t('chat.conversations')}</span>
            </div>
            <div className="sidebar-conversations-list">
              {conversations.length === 0 ? (
                <div className="sidebar-conversations-empty">{t('chat.noConversations')}</div>
              ) : (
                conversations.map((c) => (
                  <div
                    key={c.id}
                    className={`sidebar-conversation-item ${c.id === activeConversationId ? 'active' : ''}`}
                    onClick={() => {
                      setActiveConversation(c.id);
                      setMobileMenuOpen(false);
                      navigate('/chat');
                    }}
                  >
                    <MessageSquare size={13} strokeWidth={1.75} />
                    <span className="sidebar-conversation-title">{c.title}</span>
                    <button
                      className="sidebar-conversation-delete"
                      onClick={(e) => {
                        e.stopPropagation();
                        deleteConversation(c.id);
                      }}
                    >
                      <Trash2 size={11} />
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </nav>

        <div className="sidebar-footer">
          <div className="sidebar-user">
            <div className="sidebar-user-avatar">
              {user?.email?.charAt(0).toUpperCase() || 'U'}
            </div>
            <div className="sidebar-user-info">
              <span className="sidebar-user-email">{user?.email || 'User'}</span>
              <span className="sidebar-user-plan">{user?.subscription_plan || 'free'}</span>
            </div>
          </div>
          <div className="sidebar-footer-actions">
            <button
              className="sidebar-lang-btn"
              onClick={() => setLocale(locale === 'en' ? 'zh' : 'en')}
              title={locale === 'en' ? '切换到中文' : 'Switch to English'}
            >
              {locale === 'en' ? '中文' : 'EN'}
            </button>
            <button className="sidebar-logout" onClick={logout} title={t('common.signOut')}>
              <LogOut size={16} />
            </button>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="main-content">
        {/* Mobile top bar */}
        <div className="mobile-topbar">
          <button className="mobile-menu-btn" onClick={() => setMobileMenuOpen(true)}>
            <Menu size={20} />
          </button>
          <div className="mobile-topbar-brand">
            <Zap size={14} strokeWidth={2.5} />
            <span>Nexus</span>
          </div>
          <button
            className="mobile-new-chat-btn"
            onClick={() => {
              createConversation(selectedModel);
              navigate('/chat');
            }}
          >
            <Plus size={18} />
          </button>
        </div>
        <Outlet />
      </main>
    </div>
  );
}
