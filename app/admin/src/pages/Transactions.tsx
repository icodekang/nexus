import { useI18n } from '../i18n';

export default function Transactions() {
  const { t } = useI18n();
  const transactions = [
    { id: '1', user: 'user@company.com', type: 'Purchase', plan: 'Yearly', amount: '$199', status: 'Completed', date: '2024-01-15' },
    { id: '2', user: 'john@company.com', type: 'Purchase', plan: 'Monthly', amount: '$19.9', status: 'Completed', date: '2024-01-14' },
    { id: '3', user: 'jane@company.com', type: 'Purchase', plan: 'Team', amount: '$99', status: 'Completed', date: '2024-01-14' },
    { id: '4', user: 'bob@company.com', type: 'Refund', plan: 'Monthly', amount: '-$19.9', status: 'Refunded', date: '2024-01-13' },
    { id: '5', user: 'alice@company.com', type: 'Renewal', plan: 'Monthly', amount: '$19.9', status: 'Completed', date: '2024-01-12' },
  ];

  const planColors: Record<string, string> = {
    Yearly: '#6366F1',
    Monthly: '#3B82F6',
    Team: '#F59E0B',
    Enterprise: '#EC4899',
  };

  const statusLabel = (s: string) => {
    if (s === 'Completed') return t('transactions.completed');
    if (s === 'Refunded') return t('transactions.refunded');
    return s;
  };

  const typeLabel = (s: string) => {
    if (s === 'Purchase') return t('transactions.purchase');
    if (s === 'Refund') return t('transactions.refund');
    if (s === 'Renewal') return t('transactions.renewal');
    return s;
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('transactions.title')}</h1>
          <p style={styles.pageSubtitle}>{t('transactions.subtitle')}</p>
        </div>
        <div style={styles.filters}>
          <select style={styles.select}>
            <option>{t('transactions.allTypes')}</option>
          </select>
          <select style={styles.select}>
            <option>{t('transactions.allStatus')}</option>
          </select>
        </div>
      </header>

      <div style={styles.summaryGrid}>
        {[
          { label: t('transactions.revenueToday'), value: '$318', color: '#6366F1' },
          { label: t('transactions.transactionsCount'), value: '1,234', color: '#22C55E' },
          { label: t('transactions.avgOrder'), value: '$25.6', color: '#F59E0B' },
        ].map((s, i) => (
          <div key={i} style={styles.summaryCard}>
            <div style={{ ...styles.summaryDot, backgroundColor: s.color }} />
            <span style={styles.summaryLabel}>{s.label}</span>
            <span style={styles.summaryValue}>{s.value}</span>
          </div>
        ))}
      </div>

      <div style={styles.tableCard}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={{ ...styles.th, paddingLeft: '20px' }}>{t('transactions.thUser')}</th>
              <th style={styles.th}>{t('transactions.thType')}</th>
              <th style={styles.th}>{t('transactions.thPlan')}</th>
              <th style={styles.th}>{t('transactions.thAmount')}</th>
              <th style={styles.th}>{t('transactions.thStatus')}</th>
              <th style={{ ...styles.th, paddingRight: '20px' }}>{t('transactions.thDate')}</th>
            </tr>
          </thead>
          <tbody>
            {transactions.map((tx) => (
              <tr key={tx.id} style={styles.tr}>
                <td style={{ ...styles.td, paddingLeft: '20px' }}>
                  <div style={styles.userCell}>
                    <div style={styles.userAvatar}>{tx.user.charAt(0).toUpperCase()}</div>
                    <span style={styles.email}>{tx.user}</span>
                  </div>
                </td>
                <td style={styles.td}>
                  <span style={styles.type}>{typeLabel(tx.type)}</span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.planBadge,
                    color: planColors[tx.plan] || '#A1A1AA',
                    backgroundColor: `${planColors[tx.plan] || '#A1A1AA'}12`,
                  }}>
                    {tx.plan}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.amount,
                    color: tx.amount.startsWith('-') ? '#EF4444' : '#18181B',
                  }}>
                    {tx.amount}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.status,
                    color: tx.status === 'Completed' ? '#22C55E' : '#F59E0B',
                  }}>
                    <span style={{
                      ...styles.statusDot,
                      backgroundColor: tx.status === 'Completed' ? '#22C55E' : '#F59E0B',
                    }} />
                    {statusLabel(tx.status)}
                  </span>
                </td>
                <td style={{ ...styles.td, paddingRight: '20px' }}>
                  <span style={styles.date}>{tx.date}</span>
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
  filters: { display: 'flex', gap: '8px' },
  select: {
    padding: '8px 28px 8px 12px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    fontSize: '12px',
    backgroundColor: '#FFFFFF',
    fontFamily: "'DM Sans', sans-serif",
    cursor: 'pointer',
    appearance: 'none',
    backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2371717A' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E")`,
    backgroundRepeat: 'no-repeat',
    backgroundPosition: 'right 8px center',
    color: '#52525B',
  },
  summaryGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    gap: '14px',
    marginBottom: '24px',
  },
  summaryCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    padding: '18px 20px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
  },
  summaryDot: {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
  },
  summaryLabel: {
    fontSize: '12px',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
  summaryValue: {
    fontSize: '20px',
    fontWeight: '700',
    color: '#18181B',
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.02em',
  },
  tableCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    overflow: 'hidden',
  },
  table: { width: '100%', borderCollapse: 'collapse' },
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
  type: { color: '#71717A' },
  planBadge: {
    fontSize: '11px',
    fontWeight: '500',
    padding: '3px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  amount: {
    fontWeight: '600',
    fontSize: '13px',
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
};
