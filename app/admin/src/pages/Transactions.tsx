export default function Transactions() {
  const transactions = [
    { id: '1', user: 'user@example.com', type: 'Subscription Purchase', plan: 'Yearly', amount: '$199.00', status: 'Completed', date: '2024-01-15 10:30' },
    { id: '2', user: 'john@example.com', type: 'Subscription Purchase', plan: 'Monthly', amount: '$19.90', status: 'Completed', date: '2024-01-14 15:22' },
    { id: '3', user: 'jane@example.com', type: 'Subscription Purchase', plan: 'Team', amount: '$99.00', status: 'Completed', date: '2024-01-14 09:15' },
    { id: '4', user: 'bob@example.com', type: 'Refund', plan: 'Monthly', amount: '-$19.90', status: 'Refunded', date: '2024-01-13 11:45' },
    { id: '5', user: 'alice@example.com', type: 'Subscription Renewal', plan: 'Monthly', amount: '$19.90', status: 'Completed', date: '2024-01-12 08:30' },
  ];

  return (
    <div>
      <div style={styles.header}>
        <h1 style={styles.pageTitle}>Transactions</h1>
        <div style={styles.filters}>
          <select style={styles.select}>
            <option>All Types</option>
            <option>Subscription Purchase</option>
            <option>Subscription Renewal</option>
            <option>Refund</option>
          </select>
          <select style={styles.select}>
            <option>All Status</option>
            <option>Completed</option>
            <option>Pending</option>
            <option>Refunded</option>
          </select>
        </div>
      </div>
      
      <div style={styles.summary}>
        <div style={styles.summaryCard}>
          <div style={styles.summaryLabel}>Total Revenue (Today)</div>
          <div style={styles.summaryValue}>$318.80</div>
        </div>
        <div style={styles.summaryCard}>
          <div style={styles.summaryLabel}>Total Transactions</div>
          <div style={styles.summaryValue}>1,234</div>
        </div>
        <div style={styles.summaryCard}>
          <div style={styles.summaryLabel}>Average Order Value</div>
          <div style={styles.summaryValue}>$25.60</div>
        </div>
      </div>
      
      <div style={styles.table}>
        <div style={styles.tableHeader}>
          <div style={styles.tableCell}>User</div>
          <div style={styles.tableCell}>Type</div>
          <div style={styles.tableCell}>Plan</div>
          <div style={styles.tableCell}>Amount</div>
          <div style={styles.tableCell}>Status</div>
          <div style={styles.tableCell}>Date</div>
        </div>
        {transactions.map((tx) => (
          <div key={tx.id} style={styles.tableRow}>
            <div style={styles.tableCell}>
              <div style={styles.userEmail}>{tx.user}</div>
            </div>
            <div style={styles.tableCell}>{tx.type}</div>
            <div style={styles.tableCell}>
              <span style={styles.planBadge}>{tx.plan}</span>
            </div>
            <div style={styles.tableCell}>
              <span style={[styles.amount, tx.amount.startsWith('-') && styles.amountNegative]}>
                {tx.amount}
              </span>
            </div>
            <div style={styles.tableCell}>
              <span style={[styles.statusBadge, tx.status === 'Completed' ? styles.statusCompleted : styles.statusRefunded]}>
                {tx.status}
              </span>
            </div>
            <div style={styles.tableCell}>{tx.date}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
  },
  pageTitle: {
    fontSize: '28px',
    fontWeight: '700',
    color: '#1d1d1f',
  },
  filters: {
    display: 'flex',
    gap: '12px',
  },
  select: {
    padding: '10px 16px',
    borderRadius: '8px',
    border: '1px solid #e5e5e7',
    fontSize: '14px',
    backgroundColor: '#fff',
    cursor: 'pointer',
  },
  summary: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    gap: '20px',
    marginBottom: '24px',
  },
  summaryCard: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    padding: '20px',
  },
  summaryLabel: {
    fontSize: '14px',
    color: '#86868b',
    marginBottom: '8px',
  },
  summaryValue: {
    fontSize: '24px',
    fontWeight: '700',
    color: '#1d1d1f',
  },
  table: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    overflow: 'hidden',
  },
  tableHeader: {
    display: 'grid',
    gridTemplateColumns: '2fr 1.5fr 1fr 1fr 1fr 1.5fr',
    padding: '16px 20px',
    backgroundColor: '#f5f5f7',
    fontSize: '12px',
    fontWeight: '600',
    color: '#86868b',
    textTransform: 'uppercase',
  },
  tableRow: {
    display: 'grid',
    gridTemplateColumns: '2fr 1.5fr 1fr 1fr 1fr 1.5fr',
    padding: '16px 20px',
    borderBottom: '1px solid #f5f5f7',
    alignItems: 'center',
  },
  tableCell: {
    fontSize: '14px',
    color: '#1d1d1f',
  },
  userEmail: {
    fontWeight: '500',
  },
  planBadge: {
    backgroundColor: '#e8f5ef',
    color: '#10a37f',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
  },
  amount: {
    fontWeight: '600',
    color: '#16a34a',
  },
  amountNegative: {
    color: '#dc2626',
  },
  statusBadge: {
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
  },
  statusCompleted: {
    backgroundColor: '#dcfce7',
    color: '#16a34a',
  },
  statusRefunded: {
    backgroundColor: '#fef3c7',
    color: '#d97706',
  },
};
