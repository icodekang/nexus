/**
 * @file Dashboard - 管理员仪表盘页面
 * 展示系统关键统计数据，包括用户数、订阅数、收入和 API 调用量
 * 包含收入趋势图表和最近活动时间线
 */
import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';
import { fetchDashboardStats, fetchRevenueTrends, fetchRecentActivity, type DashboardStats, type RevenueTrend, type RecentActivity } from '../api/admin';

type TimeRange = '30d' | '7d';

export default function Dashboard() {
  const { t } = useI18n();
  const [timeRange, setTimeRange] = useState<TimeRange>('7d');
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [chartData, setChartData] = useState<RevenueTrend[]>([]);
  const [activities, setActivities] = useState<RecentActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [hoveredBar, setHoveredBar] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    Promise.all([
      fetchDashboardStats(),
      fetchRevenueTrends(7),
      fetchRecentActivity(),
    ]).then(([statsData, trendData, activityData]) => {
      if (cancelled) return;
      setStats(statsData);
      setChartData(trendData);
      setActivities(activityData);
    }).catch(() => {}).finally(() => {
      if (!cancelled) setLoading(false);
    });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    let cancelled = false;
    const days = timeRange === '7d' ? 7 : 30;
    fetchRevenueTrends(days)
      .then((data) => { if (!cancelled) setChartData(data); })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [timeRange]);

  const maxValue = chartData.length > 0 ? Math.max(...chartData.map((d) => d.value)) : 1;

  const statMetas = [
    { title: t('dashboard.totalUsers'), color: '#6366F1', value: stats ? String(stats.total_users) : '-', change: '' },
    { title: t('dashboard.activeSubscriptions'), color: '#22C55E', value: stats ? String(stats.active_subscriptions) : '-', change: '' },
    { title: t('dashboard.monthlyRevenue'), color: '#F59E0B', value: stats ? `$${stats.total_revenue.toLocaleString()}` : '-', change: '' },
    { title: t('dashboard.apiCallsToday'), color: '#EC4899', value: stats ? String(stats.api_calls_today) : '-', change: '' },
  ];

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('dashboard.title')}</h1>
          <p style={styles.pageSubtitle}>{loading ? 'Loading...' : t('dashboard.subtitle')}</p>
        </div>
        <select
          style={styles.select}
          value={timeRange}
          onChange={(e) => setTimeRange(e.target.value as TimeRange)}
        >
          <option value="7d">{t('dashboard.last7Days')}</option>
          <option value="30d">{t('dashboard.last30Days')}</option>
        </select>
      </header>

      <div style={styles.statsGrid}>
        {statMetas.map((stat, i) => (
          <div key={i} style={styles.statCard}>
            <div style={styles.statTop}>
              <div style={{ ...styles.statDot, backgroundColor: stat.color }} />
              <span style={styles.statTitle}>{stat.title}</span>
            </div>
            <div style={styles.statBottom}>
              <span style={styles.statValue}>{stat.value}</span>
              {stat.change && <span style={styles.statChange}>{stat.change}</span>}
            </div>
          </div>
        ))}
      </div>

      <div style={styles.mainGrid}>
        {/* 收入概览图表 */}
        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <h2 style={styles.cardTitle}>{t('dashboard.revenueOverview')}</h2>
            <span style={styles.cardBadge}>
              {timeRange === '30d' ? t('dashboard.last30Days') : t('dashboard.last7Days')}
            </span>
          </div>
          <div style={styles.chart}>
            {chartData.map((d, i) => (
              <div
                key={i}
                style={styles.chartCol}
                onMouseEnter={() => setHoveredBar(i)}
                onMouseLeave={() => setHoveredBar(null)}
              >
                <div style={styles.chartBarTrack}>
                  <div
                    style={{
                      ...styles.chartBarFill,
                      height: maxValue > 0 ? `${Math.max((d.value / maxValue) * 100, 2)}%` : '2%',
                      backgroundColor: d.value > 0
                        ? (hoveredBar === i ? '#4F46E5' : '#6366F1')
                        : '#E7E5E4',
                      transform: hoveredBar === i ? 'scaleX(1.1)' : 'scaleX(1)',
                    }}
                  />
                </div>
                {/* 悬浮提示 */}
                {hoveredBar === i && (
                  <div style={styles.tooltip}>
                    ${d.value.toFixed(2)}
                    <div style={styles.tooltipDate}>{d.date}</div>
                  </div>
                )}
                <span style={{
                  ...styles.chartLabel,
                  color: hoveredBar === i ? '#18181B' : '#A1A1AA',
                  fontWeight: hoveredBar === i ? 600 : 400,
                }}>
                  {d.label}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* 最近活动列表 */}
        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <h2 style={styles.cardTitle}>{t('dashboard.recentActivity')}</h2>
          </div>
          <div style={styles.activityList}>
            {activities.length === 0 && (
              <div style={styles.emptyActivity}>No recent activity</div>
            )}
            {activities.map((item, i) => (
              <div key={i} style={styles.activityItem}>
                <div style={{
                  ...styles.activityAvatar,
                  backgroundColor: item.action_type === 'api_call' ? '#EEF2FF' : '#F0FDF4',
                  color: item.action_type === 'api_call' ? '#6366F1' : '#22C55E',
                }}>
                  {item.user_email.charAt(0).toUpperCase()}
                </div>
                <div style={styles.activityContent}>
                  <span style={styles.activityAction}>{item.description}</span>
                  <span style={styles.activityUser}>{item.user_email}</span>
                </div>
                <span style={styles.activityTime}>{item.time_ago}</span>
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
    height: '140px',
    paddingTop: '8px',
    position: 'relative',
  },
  chartCol: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    flex: 1,
    height: '100%',
    justifyContent: 'flex-end',
    gap: '8px',
    position: 'relative',
    cursor: 'pointer',
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
    transition: 'height 0.3s ease, background-color 0.15s ease, transform 0.15s ease',
    transformOrigin: 'bottom center',
  },
  tooltip: {
    position: 'absolute',
    bottom: '100%',
    left: '50%',
    transform: 'translateX(-50%)',
    marginBottom: '8px',
    backgroundColor: '#18181B',
    color: '#FFFFFF',
    fontSize: '13px',
    fontWeight: '600',
    fontFamily: "'DM Sans', sans-serif",
    padding: '6px 10px',
    borderRadius: '8px',
    whiteSpace: 'nowrap',
    zIndex: 10,
    boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
  },
  tooltipDate: {
    fontSize: '10px',
    fontWeight: '400',
    color: '#A1A1AA',
    marginTop: '2px',
    textAlign: 'center',
  },
  chartLabel: {
    fontSize: '10px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'color 0.15s ease, font-weight 0.15s ease',
  },
  activityList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  },
  emptyActivity: {
    fontSize: '13px',
    color: '#A1A1AA',
    textAlign: 'center',
    padding: '32px 0',
    fontFamily: "'DM Sans', sans-serif",
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
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '12px',
    fontWeight: '600',
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
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
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
