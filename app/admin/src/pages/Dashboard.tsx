import { useI18n } from '../i18n';

export default function Dashboard() {
  const { t } = useI18n();

  const stats = [
    { title: t('dashboard.totalUsers'), value: '1,234', change: '+12%', color: '#6366F1' },
    { title: t('dashboard.activeSubscriptions'), value: '892', change: '+8%', color: '#22C55E' },
    { title: t('dashboard.monthlyRevenue'), value: '$45,678', change: '+23%', color: '#F59E0B' },
    { title: t('dashboard.apiCallsToday'), value: '89,234', change: '+5%', color: '#EC4899' },
  ];

  const chartData = [
    { month: 'Jul', value: 28000 },
    { month: 'Aug', value: 32000 },
    { month: 'Sep', value: 35000 },
    { month: 'Oct', value: 38000 },
    { month: 'Nov', value: 42000 },
    { month: 'Dec', value: 45678 },
  ];

  const maxValue = Math.max(...chartData.map((d) => d.value));

  const activities = [
    { user: 'user@company.com', action: t('dashboard.purchasedPlan', { plan: 'Yearly' }), time: '2m' },
    { user: 'john@company.com', action: t('dashboard.madeApiCall'), time: '5m' },
    { user: 'jane@company.com', action: t('dashboard.upgradedTo', { plan: 'Team' }), time: '12m' },
    { user: 'bob@company.com', action: t('dashboard.subscriptionExpired'), time: '1h' },
  ];

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('dashboard.title')}</h1>
          <p style={styles.pageSubtitle}>{t('dashboard.subtitle')}</p>
        </div>
        <select style={styles.select}>
          <option>{t('dashboard.last30Days')}</option>
          <option>{t('dashboard.last7Days')}</option>
        </select>
      </header>

      <div style={styles.statsGrid}>
        {stats.map((stat, i) => (
          <div key={i} style={styles.statCard}>
            <div style={styles.statTop}>
              <div style={{ ...styles.statDot, backgroundColor: stat.color }} />
              <span style={styles.statTitle}>{stat.title}</span>
            </div>
            <div style={styles.statBottom}>
              <span style={styles.statValue}>{stat.value}</span>
              <span style={styles.statChange}>{stat.change}</span>
            </div>
          </div>
        ))}
      </div>

      <div style={styles.mainGrid}>
        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <h2 style={styles.cardTitle}>{t('dashboard.revenueOverview')}</h2>
            <span style={styles.cardBadge}>2024</span>
          </div>
          <div style={styles.chart}>
            {chartData.map((d, i) => (
              <div key={i} style={styles.chartCol}>
                <div style={styles.chartBarTrack}>
                  <div
                    style={{
                      ...styles.chartBarFill,
                      height: `${(d.value / maxValue) * 100}%`,
                    }}
                  />
                </div>
                <span style={styles.chartLabel}>{d.month}</span>
              </div>
            ))}
          </div>
        </div>

        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <h2 style={styles.cardTitle}>{t('dashboard.recentActivity')}</h2>
          </div>
          <div style={styles.activityList}>
            {activities.map((item, i) => (
              <div key={i} style={styles.activityItem}>
                <div style={styles.activityAvatar}>
                  {item.user.charAt(0).toUpperCase()}
                </div>
                <div style={styles.activityContent}>
                  <span style={styles.activityAction}>{item.action}</span>
                  <span style={styles.activityUser}>{item.user}</span>
                </div>
                <span style={styles.activityTime}>{t('dashboard.timeAgo', { time: item.time })}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    maxWidth: '1200px',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-end',
    marginBottom: '28px',
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
  select: {
    padding: '8px 32px 8px 12px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    fontSize: '12px',
    backgroundColor: '#FFFFFF',
    fontFamily: "'DM Sans', sans-serif",
    cursor: 'pointer',
    appearance: 'none',
    backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%2371717A' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E")`,
    backgroundRepeat: 'no-repeat',
    backgroundPosition: 'right 10px center',
    color: '#52525B',
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '14px',
    marginBottom: '20px',
  },
  statCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    padding: '18px 20px',
    border: 'none',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  statTop: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  statDot: {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
  },
  statTitle: {
    fontSize: '12px',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
  statBottom: {
    display: 'flex',
    alignItems: 'baseline',
    justifyContent: 'space-between',
  },
  statValue: {
    fontSize: '22px',
    fontWeight: '700',
    color: '#18181B',
    fontFamily: "'Instrument Sans', sans-serif",
    letterSpacing: '-0.02em',
  },
  statChange: {
    fontSize: '11px',
    fontWeight: '500',
    color: '#22C55E',
    backgroundColor: 'rgba(34, 197, 94, 0.08)',
    padding: '2px 8px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  mainGrid: {
    display: 'grid',
    gridTemplateColumns: '2fr 1fr',
    gap: '14px',
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    padding: '24px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '20px',
  },
  cardTitle: {
    fontSize: '14px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  cardBadge: {
    fontSize: '11px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  chart: {
    display: 'flex',
    alignItems: 'flex-end',
    justifyContent: 'space-between',
    height: '120px',
    paddingTop: '8px',
  },
  chartCol: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    flex: 1,
    height: '100%',
    justifyContent: 'flex-end',
    gap: '8px',
  },
  chartBarTrack: {
    width: '28px',
    height: '100%',
    backgroundColor: '#F5F5F4',
    borderRadius: '6px',
    display: 'flex',
    alignItems: 'flex-end',
    overflow: 'hidden',
  },
  chartBarFill: {
    width: '100%',
    backgroundColor: '#6366F1',
    borderRadius: '6px',
    minHeight: '4px',
    transition: 'height 0.3s ease',
  },
  chartLabel: {
    fontSize: '10px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  activityList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  },
  activityItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '10px 0',
  },
  activityAvatar: {
    width: '30px',
    height: '30px',
    borderRadius: '8px',
    backgroundColor: '#F5F5F4',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '12px',
    fontWeight: '600',
    color: '#71717A',
    fontFamily: "'Instrument Sans', sans-serif",
    flexShrink: 0,
  },
  activityContent: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    gap: '1px',
    minWidth: 0,
  },
  activityAction: {
    fontSize: '12px',
    fontWeight: '500',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  activityUser: {
    fontSize: '11px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  activityTime: {
    fontSize: '10px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
    flexShrink: 0,
  },
};
