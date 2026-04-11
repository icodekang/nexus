import { useState } from 'react';
import { useI18n } from '../i18n';

export default function Users() {
  const { t } = useI18n();
  const [searchQuery, setSearchQuery] = useState('');

  const users = [
    { id: '1', email: 'user@company.com', phone: '+86 138****8888', plan: 'Yearly', status: 'Active', created: '2024-01-01' },
    { id: '2', email: 'john@company.com', phone: '+86 139****6666', plan: 'Monthly', status: 'Active', created: '2024-01-05' },
    { id: '3', email: 'jane@company.com', phone: '+86 137****5555', plan: 'Team', status: 'Active', created: '2024-01-10' },
    { id: '4', email: 'bob@company.com', phone: '+86 136****4444', plan: 'None', status: 'Inactive', created: '2023-12-01' },
    { id: '5', email: 'alice@company.com', phone: '+86 135****3333', plan: 'Enterprise', status: 'Active', created: '2024-01-15' },
  ];

  const planColors: Record<string, string> = {
    Yearly: '#6366F1',
    Monthly: '#3B82F6',
    Team: '#F59E0B',
    Enterprise: '#EC4899',
    None: '#A1A1AA',
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('users.title')}</h1>
          <p style={styles.pageSubtitle}>{t('users.count', { count: users.length })}</p>
        </div>
        <div style={styles.headerActions}>
          <div style={styles.searchBox}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="11" cy="11" r="8" />
              <path d="M21 21l-4.35-4.35" />
            </svg>
            <input
              type="text"
              placeholder={t('users.searchPlaceholder')}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={styles.searchInput}
            />
          </div>
          <button style={styles.addBtn}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            {t('users.addUser')}
          </button>
        </div>
      </header>

      <div style={styles.tableCard}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={{ ...styles.th, paddingLeft: '20px' }}>{t('users.thUser')}</th>
              <th style={styles.th}>{t('users.thPhone')}</th>
              <th style={styles.th}>{t('users.thPlan')}</th>
              <th style={styles.th}>{t('users.thStatus')}</th>
              <th style={styles.th}>{t('users.thCreated')}</th>
              <th style={{ ...styles.th, paddingRight: '20px', textAlign: 'right' }}></th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.id} style={styles.tr}>
                <td style={{ ...styles.td, paddingLeft: '20px' }}>
                  <div style={styles.userCell}>
                    <div style={styles.userAvatar}>{user.email.charAt(0).toUpperCase()}</div>
                    <span style={styles.email}>{user.email}</span>
                  </div>
                </td>
                <td style={styles.td}>
                  <span style={styles.phone}>{user.phone}</span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.planBadge,
                    color: planColors[user.plan] || '#A1A1AA',
                    backgroundColor: `${planColors[user.plan] || '#A1A1AA'}12`,
                  }}>
                    {user.plan}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.status,
                    color: user.status === 'Active' ? '#22C55E' : '#EF4444',
                  }}>
                    <span style={{
                      ...styles.statusDot,
                      backgroundColor: user.status === 'Active' ? '#22C55E' : '#EF4444',
                    }} />
                    {user.status === 'Active' ? t('common.active') : t('common.inactive')}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={styles.date}>{user.created}</span>
                </td>
                <td style={{ ...styles.td, paddingRight: '20px', textAlign: 'right' }}>
                  <button style={styles.actionBtn}>{t('common.edit')}</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: { maxWidth: '1200px' },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-end',
    marginBottom: '24px',
  },
  pageTitle: {
    fontSize: '24px',
    fontWeight: '700',
    color: '#18181B',
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.02em',
  },
  pageSubtitle: {
    fontSize: '13px',
    color: '#71717A',
    marginTop: '4px',
    fontFamily: "'DM Sans', sans-serif",
  },
  headerActions: {
    display: 'flex',
    gap: '10px',
  },
  searchBox: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px 12px',
    backgroundColor: '#FFFFFF',
    borderRadius: '10px',
    border: '1px solid #E7E5E4',
    width: '200px',
    boxShadow: '0 1px 2px rgba(0,0,0,0.04)',
  },
  searchInput: {
    flex: 1,
    border: 'none',
    outline: 'none',
    fontSize: '12px',
    fontFamily: "'DM Sans', sans-serif",
    backgroundColor: 'transparent',
    color: '#18181B',
  },
  addBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 14px',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    border: 'none',
    borderRadius: '10px',
    fontSize: '12px',
    fontWeight: '500',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
  tableCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    overflow: 'hidden',
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse',
  },
  th: {
    padding: '12px 16px',
    textAlign: 'left',
    fontSize: '11px',
    fontWeight: '500',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    fontFamily: "'DM Sans', sans-serif",
    borderBottom: '1px solid #F5F5F4',
  },
  tr: {
    borderBottom: '1px solid #F5F5F4',
    transition: 'background 0.1s ease',
  },
  td: {
    padding: '14px 16px',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
  },
  userCell: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  userAvatar: {
    width: '28px',
    height: '28px',
    borderRadius: '8px',
    backgroundColor: '#F5F5F4',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '11px',
    fontWeight: '600',
    color: '#71717A',
    fontFamily: "'Instrument Sans', sans-serif",
    flexShrink: 0,
  },
  email: {
    fontWeight: '500',
    color: '#18181B',
  },
  phone: {
    color: '#71717A',
  },
  planBadge: {
    fontSize: '11px',
    fontWeight: '500',
    padding: '3px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  status: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '12px',
    fontWeight: '500',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: {
    width: '5px',
    height: '5px',
    borderRadius: '50%',
    flexShrink: 0,
  },
  date: {
    color: '#A1A1AA',
    fontSize: '12px',
  },
  actionBtn: {
    padding: '5px 12px',
    backgroundColor: 'transparent',
    border: '1px solid #E7E5E4',
    borderRadius: '8px',
    fontSize: '11px',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.1s ease',
  },
};
