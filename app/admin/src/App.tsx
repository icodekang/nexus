import { BrowserRouter, Routes, Route, Navigate, Link, useLocation } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import Users from './pages/Users';
import Providers from './pages/Providers';
import Models from './pages/Models';
import Transactions from './pages/Transactions';

function Navigation() {
  const location = useLocation();
  
  const navItems = [
    { path: '/dashboard', label: 'Dashboard', icon: '📊' },
    { path: '/users', label: 'Users', icon: '👥' },
    { path: '/providers', label: 'Providers', icon: '☁️' },
    { path: '/models', label: 'Models', icon: '🤖' },
    { path: '/transactions', label: 'Transactions', icon: '💳' },
  ];
  
  return (
    <nav style={styles.nav}>
      <div style={styles.navBrand}>
        <span style={styles.navLogo}>N</span>
        <span style={styles.navTitle}>Nexus Admin</span>
      </div>
      <div style={styles.navItems}>
        {navItems.map((item) => (
          <Link
            key={item.path}
            to={item.path}
            style={[
              styles.navItem,
              location.pathname === item.path && styles.navItemActive,
            ]}
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </Link>
        ))}
      </div>
    </nav>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <div style={styles.layout}>
        <Navigation />
        <main style={styles.main}>
          <Routes>
            <Route path="/" element={<Navigate to="/dashboard" replace />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/users" element={<Users />} />
            <Route path="/providers" element={<Providers />} />
            <Route path="/models" element={<Models />} />
            <Route path="/transactions" element={<Transactions />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}

const styles: Record<string, React.CSSProperties> = {
  layout: {
    display: 'flex',
    minHeight: '100vh',
    backgroundColor: '#f5f5f7',
  },
  nav: {
    width: '240px',
    backgroundColor: '#1d1d1f',
    color: '#fff',
    display: 'flex',
    flexDirection: 'column',
  },
  navBrand: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '20px 24px',
    borderBottom: '1px solid #333',
  },
  navLogo: {
    width: '32px',
    height: '32px',
    backgroundColor: '#10A37F',
    borderRadius: '8px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontWeight: 'bold',
    fontSize: '18px',
  },
  navTitle: {
    fontSize: '16px',
    fontWeight: '600',
  },
  navItems: {
    display: 'flex',
    flexDirection: 'column',
    padding: '16px 12px',
    gap: '4px',
  },
  navItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '12px 16px',
    borderRadius: '8px',
    color: '#a1a1a6',
    textDecoration: 'none',
    fontSize: '14px',
    fontWeight: '500',
    transition: 'all 0.2s',
  },
  navItemActive: {
    backgroundColor: '#10A37F',
    color: '#fff',
  },
  main: {
    flex: 1,
    padding: '32px',
    overflow: 'auto',
  },
};
