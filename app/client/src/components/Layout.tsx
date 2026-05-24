/**
 * @file Layout - 客户端应用布局组件
 * 包含顶部导航栏、侧边栏（桌面端）/抽屉式（移动端）
 * 管理和展示对话列表、用户信息
 */
import { useState, useRef, useEffect } from 'react';
import { NavLink, Outlet, useLocation, useNavigate } from 'react-router-dom';
import { MessageSquare, Key, BookOpen, Layers, LogOut, Zap, Menu, X, CreditCard, Plus, Trash2 } from 'lucide-react';
import { useAuthStore } from '../stores/authStore';
import { useChatStore } from '../stores/chatStore';
import { useI18n } from '../i18n';
import './Layout.css';

/**
 * Layout - 布局主组件
 * @description 响应式侧边栏 + 主内容区，底部用户信息和操作
 */
export default function Layout() {
  const { user, logout } = useAuthStore();
  const {
    conversations,
    activeConversationId,
    setActiveConversation,
    deleteConversation,
    createConversation,
    selectedModelId,
  } = useChatStore();
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const sidebarRef = useRef<HTMLDivElement>(null);
  const { t, locale, setLocale } = useI18n();

  const navItems = [
    { to: '/chat', label: t('layout.chat'), icon: MessageSquare },
    { to: '/models', label: t('layout.models'), icon: Layers },
    { to: '/keys', label: t('layout.apiKeys'), icon: Key },
    { to: '/subscription', label: t('layout.subscription'), icon: CreditCard },
    { to: '/guide', label: t('layout.guide'), icon: BookOpen },
  ];

  useEffect(() => {
    setMobileMenuOpen(false);
  }, [location.pathname]);

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

  const handleNewChat = () => {
    const modelId = selectedModelId || 'gpt-4o-mini';
    const id = createConversation(modelId);
    setActiveConversation(id);
    navigate('/chat');
  };

  const handleSelectConv = (id: string) => {
    setActiveConversation(id);
    navigate('/chat');
  };

  const handleDeleteConv = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    deleteConversation(id);
  };

  const recentConversations = conversations.slice(0, 20);

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

        <div className="sidebar-new-chat-wrapper">
          <button className="sidebar-new-chat-btn" onClick={handleNewChat}>
            <Plus size={16} />
            <span>{t('chat.newChat')}</span>
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
        </nav>

        <div className="sidebar-conversations">
          <div className="sidebar-conversations-divider">
            <span className="sidebar-conversations-divider-label">{t('chat.conversations')}</span>
          </div>
          <div className="sidebar-conversations-list">
            {recentConversations.length === 0 ? (
              <div className="sidebar-conversations-empty">{t('chat.noConversations')}</div>
            ) : (
              recentConversations.map((conv) => (
                <div
                  key={conv.id}
                  className={`sidebar-conversation-item ${conv.id === activeConversationId ? 'active' : ''}`}
                  onClick={() => handleSelectConv(conv.id)}
                >
                  <span className="sidebar-conversation-title">
                    {conv.title || t('chat.newConversation')}
                  </span>
                  <button
                    className="sidebar-conversation-delete"
                    onClick={(e) => handleDeleteConv(e, conv.id)}
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="sidebar-footer">
          <div className="sidebar-user">
            <div className="sidebar-user-avatar">
              {user?.email?.charAt(0).toUpperCase() || 'U'}
            </div>
            <div className="sidebar-user-info">
              <span className="sidebar-user-email">{user?.email || t('common.user')}</span>
              <span className="sidebar-user-plan">{user?.subscription_plan || t('common.free')}</span>
            </div>
          </div>
          <div className="sidebar-footer-actions">
            <button
              className="sidebar-lang-btn"
              onClick={() => setLocale(locale === 'en' ? 'zh' : 'en')}
              title={t('common.switchLang')}
            >
              {locale === 'en' ? '中文' : 'EN'}
            </button>
            <button className="sidebar-logout" onClick={() => { logout(); navigate('/login'); }} title={t('common.signOut')}>
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
            <span>{t('common.brandName')}</span>
          </div>
          <button
            className="mobile-new-chat-btn"
            onClick={handleNewChat}
          >
            <Plus size={18} />
          </button>
        </div>
        <Outlet />
      </main>
    </div>
  );
}
