export default function Dashboard() {
  return (
    <div>
      <h1 style={styles.pageTitle}>Dashboard</h1>
      
      <div style={styles.statsGrid}>
        <StatCard title="Total Users" value="1,234" icon="👥" trend="+12%" />
        <StatCard title="Active Subscriptions" value="892" icon="✅" trend="+8%" />
        <StatCard title="Monthly Revenue" value="$45,678" icon="💰" trend="+23%" />
        <StatCard title="API Calls Today" value="89,234" icon="📊" trend="+5%" />
      </div>
      
      <div style={styles.chartSection}>
        <h2 style={styles.sectionTitle}>Revenue Overview</h2>
        <div style={styles.chartPlaceholder}>
          <p style={styles.chartText}>Chart visualization would go here</p>
        </div>
      </div>
      
      <div style={styles.recentSection}>
        <h2 style={styles.sectionTitle}>Recent Transactions</h2>
        <div style={styles.table}>
          <div style={styles.tableHeader}>
            <div style={styles.tableCell}>User</div>
            <div style={styles.tableCell}>Plan</div>
            <div style={styles.tableCell}>Amount</div>
            <div style={styles.tableCell}>Date</div>
          </div>
          <TransactionRow user="user@example.com" plan="Yearly" amount="$199" date="2024-01-15" />
          <TransactionRow user="john@example.com" plan="Monthly" amount="$19.9" date="2024-01-14" />
          <TransactionRow user="jane@example.com" plan="Team" amount="$99" date="2024-01-14" />
        </div>
      </div>
    </div>
  );
}

function StatCard({ title, value, icon, trend }: { title: string; value: string; icon: string; trend: string }) {
  return (
    <div style={styles.statCard}>
      <div style={styles.statIcon}>{icon}</div>
      <div style={styles.statContent}>
        <div style={styles.statTitle}>{title}</div>
        <div style={styles.statValue}>{value}</div>
        <div style={styles.statTrend}>{trend}</div>
      </div>
    </div>
  );
}

function TransactionRow({ user, plan, amount, date }: { user: string; plan: string; amount: string; date: string }) {
  return (
    <div style={styles.tableRow}>
      <div style={styles.tableCell}>{user}</div>
      <div style={styles.tableCell}>
        <span style={styles.planBadge}>{plan}</span>
      </div>
      <div style={styles.tableCell}>{amount}</div>
      <div style={styles.tableCell}>{date}</div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  pageTitle: {
    fontSize: '28px',
    fontWeight: '700',
    color: '#1d1d1f',
    marginBottom: '24px',
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '20px',
    marginBottom: '32px',
  },
  statCard: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    padding: '20px',
    display: 'flex',
    alignItems: 'flex-start',
    gap: '16px',
  },
  statIcon: {
    fontSize: '32px',
  },
  statContent: {
    flex: 1,
  },
  statTitle: {
    fontSize: '14px',
    color: '#86868b',
    marginBottom: '4px',
  },
  statValue: {
    fontSize: '24px',
    fontWeight: '700',
    color: '#1d1d1f',
  },
  statTrend: {
    fontSize: '12px',
    color: '#16a34a',
    marginTop: '4px',
  },
  chartSection: {
    marginBottom: '32px',
  },
  sectionTitle: {
    fontSize: '18px',
    fontWeight: '600',
    color: '#1d1d1f',
    marginBottom: '16px',
  },
  chartPlaceholder: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    padding: '60px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  chartText: {
    color: '#86868b',
    fontSize: '14px',
  },
  recentSection: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    padding: '20px',
  },
  table: {
    width: '100%',
  },
  tableHeader: {
    display: 'grid',
    gridTemplateColumns: '2fr 1fr 1fr 1fr',
    padding: '12px 16px',
    borderBottom: '1px solid #f5f5f7',
    fontSize: '12px',
    fontWeight: '600',
    color: '#86868b',
    textTransform: 'uppercase',
  },
  tableRow: {
    display: 'grid',
    gridTemplateColumns: '2fr 1fr 1fr 1fr',
    padding: '16px',
    borderBottom: '1px solid #f5f5f7',
    fontSize: '14px',
    color: '#1d1d1f',
  },
  tableCell: {
    display: 'flex',
    alignItems: 'center',
  },
  planBadge: {
    backgroundColor: '#e8f5ef',
    color: '#10a37f',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
  },
};
