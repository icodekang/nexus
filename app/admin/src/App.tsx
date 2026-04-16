/**
 * @file App - 管理员应用根组件
 * 整合路由、认证状态和国际化配置
 * 提供侧边栏布局和受保护的路由
 */
import { BrowserRouter, Routes, Route, Navigate, Link, useLocation, Outlet } from 'react-router-dom';
import { AuthProvider, useAuth } from './auth';
import { I18nProvider, useI18n } from './i18n';
import Dashboard from './pages/Dashboard';
import Users from './pages/Users';
import Providers from './pages/Providers';
import ProviderKeys from './pages/ProviderKeys';
import Models from './pages/Models';
import Transactions from './pages/Transactions';
import BrowserAccounts from './pages/BrowserAccounts';
import AuthCallback from './pages/AuthCallback';
import Login from './pages/Login';

/**
 * ProtectedRoute - 受保护的路由包装组件
 * @description 未登录用户重定向到登录页
 */
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

function Sidebar() {
  const location = useLocation();
  const { logout } = useAuth();
  const { t, locale, setLocale } = useI18n();

  // 导航菜单项配置
  const navItems = [
    { path: '/dashboard', label: t('sidebar.dashboard'), icon: DashboardIcon },
    { path: '/users', label: t('sidebar.users'), icon: UsersIcon },
    { path: '/providers', label: t('sidebar.providers'), icon: ProvidersIcon },
    { path: '/models', label: t('sidebar.models'), icon: ModelsIcon },
    { path: '/provider-keys', label: t('sidebar.providerKeys'), icon: ProviderKeysIcon },
    { path: '/browser-accounts', label: t('sidebar.browserAccounts'), icon: BrowserAccountsIcon },
    { path: '/transactions', label: t('sidebar.transactions'), icon: TransactionsIcon },
  ];

  return (
    <aside style={styles.sidebar}>
      <div style={styles.sidebarBrand}>
        <div style={styles.logoMark}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>
          </svg>
        </div>
        <span style={styles.brandName}>{t('common.brandName')}</span>
      </div>

      <nav style={styles.nav}>
        {/* 菜单分组标签 */}
        <div style={styles.navLabel}>{t('common.menu')}</div>
        {/* 渲染导航项，高亮当前激活项 */}
        {navItems.map((item) => {
          const active = location.pathname === item.path;
          return (
            <Link
              key={item.path}
              to={item.path}
              style={{
                ...styles.navItem,
                ...(active ? styles.navItemActive : {}),
              }}
            >
              <item.icon active={active} />
              <span>{item.label}</span>
              {active && <div style={styles.activeIndicator} />}
            </Link>
          );
        })}
      </nav>

      <div style={styles.sidebarFooter}>
        {/* 用户信息区域 */}
        <div style={styles.userInfo}>
          <div style={styles.avatar}>A</div>
          <div style={styles.userDetails}>
            <span style={styles.userName}>{t('common.admin')}</span>
            <span style={styles.userRole}>{t('common.administrator')}</span>
          </div>
        </div>
        {/* 底部操作：语言切换 + 登出 */}
        <div style={styles.footerActions}>
          <button
            onClick={() => setLocale(locale === 'en' ? 'zh' : 'en')}
            style={styles.langBtn}
            title={locale === 'en' ? '切换到中文' : 'Switch to English'}
          >
            {locale === 'en' ? '中文' : 'EN'}
          </button>
          <button onClick={logout} style={styles.logoutBtn} title={t('common.signOut')}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
              <polyline points="16 17 21 12 16 7" />
              <line x1="21" y1="12" x2="9" y2="12" />
            </svg>
          </button>
        </div>
      </div>
    </aside>
  );
}

// 各导航图标组件
function DashboardIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="9" rx="1.5" />
      <rect x="14" y="3" width="7" height="5" rx="1.5" />
      <rect x="14" y="12" width="7" height="9" rx="1.5" />
      <rect x="3" y="16" width="7" height="5" rx="1.5" />
    </svg>
  );
}

function UsersIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
      <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
      <path d="M16 3.13a4 4 0 0 1 0 7.75" />
    </svg>
  );
}

function ProvidersIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z" />
    </svg>
  );
}

function ProviderKeysIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
    </svg>
  );
}

function ModelsIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
    </svg>
  );
}

function TransactionsIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="5" width="20" height="14" rx="2" />
      <line x1="2" y1="10" x2="22" y2="10" />
    </svg>
  );
}

function BrowserAccountsIcon(_props: { active: boolean }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
    </svg>
  );
}

/**
 * Layout - 页面布局组件
 * @description 侧边栏 + 主内容区的组合布局
 */
function Layout() {
  return (
    <div style={styles.layout}>
      <Sidebar />
      <main style={styles.main}>
        <Outlet />
      </main>
    </div>
  );
}

/**
 * AppRoutes - 应用路由配置
 * @description 定义所有路由，包括公开路由和受保护路由
 */
function AppRoutes() {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route element={<Layout />}>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<ProtectedRoute><Dashboard /></ProtectedRoute>} />
        <Route path="/users" element={<ProtectedRoute><Users /></ProtectedRoute>} />
        <Route path="/providers" element={<ProtectedRoute><Providers /></ProtectedRoute>} />
        <Route path="/provider-keys" element={<ProtectedRoute><ProviderKeys /></ProtectedRoute>} />
        <Route path="/models" element={<ProtectedRoute><Models /></ProtectedRoute>} />
        <Route path="/transactions" element={<ProtectedRoute><Transactions /></ProtectedRoute>} />
        <Route path="/browser-accounts" element={<ProtectedRoute><BrowserAccounts /></ProtectedRoute>} />
      </Route>
      <Route path="/auth/callback" element={<AuthCallback />} />
    </Routes>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <I18nProvider>
        <AuthProvider>
          <AppRoutes />
        </AuthProvider>
      </I18nProvider>
    </BrowserRouter>
  );
}

const styles: Record<string, React.CSSProperties> = {
  layout: {
    display: 'flex',
    minHeight: '100vh',
    backgroundColor: '#FAFAF9',
  },
  sidebar: {
    width: '240px',
    backgroundColor: '#FFFFFF',
    borderRight: '1px solid #F0F0F0',
    display: 'flex',
    flexDirection: 'column',
    position: 'fixed',
    top: 0,
    left: 0,
    bottom: 0,
    zIndex: 100,
  },
  sidebarBrand: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '24px 20px 20px',
  },
  logoMark: {
    width: '34px',
    height: '34px',
    backgroundColor: '#6366F1',
    borderRadius: '10px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  brandName: {
    fontSize: '17px',
    fontWeight: '700',
    color: '#18181B',
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.02em',
  },
  nav: {
    flex: 1,
    padding: '8px 12px',
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
  },
  navLabel: {
    fontSize: '11px',
    fontWeight: '500',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.06em',
    padding: '12px 12px 6px',
    fontFamily: "'DM Sans', sans-serif",
  },
  navItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    padding: '10px 12px',
    borderRadius: '8px',
    textDecoration: 'none',
    fontSize: '13px',
    fontWeight: '500',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
    position: 'relative',
    transition: 'all 0.15s ease',
  },
  navItemActive: {
    backgroundColor: '#F5F5F4',
    color: '#18181B',
  },
  activeIndicator: {
    position: 'absolute',
    left: '0',
    top: '50%',
    transform: 'translateY(-50%)',
    width: '3px',
    height: '16px',
    backgroundColor: '#6366F1',
    borderRadius: '0 3px 3px 0',
  },
  sidebarFooter: {
    padding: '12px',
    borderTop: '1px solid #F0F0F0',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  userInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    padding: '8px',
    borderRadius: '8px',
  },
  avatar: {
    width: '30px',
    height: '30px',
    borderRadius: '8px',
    backgroundColor: '#6366F1',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: '#FFFFFF',
    fontSize: '12px',
    fontWeight: '600',
    fontFamily: "'Instrument Sans', sans-serif",
  },
  userDetails: {
    display: 'flex',
    flexDirection: 'column',
  },
  userName: {
    fontSize: '12px',
    fontWeight: '500',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  userRole: {
    fontSize: '10px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  footerActions: {
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
  },
  langBtn: {
    padding: '4px 8px',
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
  logoutBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '32px',
    height: '32px',
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '8px',
    color: '#A1A1AA',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  main: {
    flex: 1,
    marginLeft: '240px',
    padding: '32px 40px',
    minHeight: '100vh',
  },
};
