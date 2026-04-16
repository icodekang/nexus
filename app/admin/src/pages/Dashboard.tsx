/**
 * @file Dashboard - 管理员仪表盘页面
 * 展示系统关键统计数据，包括用户数、订阅数、收入和 API 调用量
 * 包含收入趋势图表和最近活动时间线
 */
import { useState, useEffect } from 'react';
import { useI18n } from '../i18n';
import { fetchDashboardStats, type DashboardStats } from '../api/admin';

type TimeRange = '30d' | '7d';

// 时间范围选项对应的图表数据（模拟数据，用于展示趋势）
const chartDataMap: Record<TimeRange, { label: string; value: number }[]> = {
  '30d': [
    { label: 'Jul', value: 28000 },
    { label: 'Aug', value: 32000 },
    { label: 'Sep', value: 35000 },
    { label: 'Oct', value: 38000 },
    { label: 'Nov', value: 42000 },
    { label: 'Dec', value: 45678 },
  ],
  '7d': [
    { label: 'Mon', value: 3200 },
    { label: 'Tue', value: 2800 },
    { label: 'Wed', value: 3600 },
    { label: 'Thu', value: 4100 },
    { label: 'Fri', value: 3900 },
    { label: 'Sat', value: 2100 },
    { label: 'Sun', value: 1880 },
  ],
};

// 最近活动数据（模拟数据，用于展示用户行为）
const activities = [
  { user: 'user@company.com', actionKey: 'dashboard.purchasedPlan', actionParams: { plan: 'Yearly' }, time: '2m' },
  { user: 'john@company.com', actionKey: 'dashboard.madeApiCall', time: '5m' },
  { user: 'jane@company.com', actionKey: 'dashboard.upgradedTo', actionParams: { plan: 'Team' }, time: '12m' },
  { user: 'bob@company.com', actionKey: 'dashboard.subscriptionExpired', time: '1h' },
];

/**
 * Dashboard - 管理员仪表盘主组件
 * @description 获取并展示系统统计数据、收入图表和最近活动
 */
export default function Dashboard() {
  const { t } = useI18n();
  const [timeRange, setTimeRange] = useState<TimeRange>('30d');
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  // 组件挂载时从 API 获取统计数据
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    fetchDashboardStats()
      .then((data) => {
        if (!cancelled) setStats(data);
      })
      .catch(() => {
        // 错误时保持默认值
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, []);

  // 根据选择的时间范围获取图表数据，并计算最大值用于柱状图比例
  const chartData = chartDataMap[timeRange];
  const maxValue = Math.max(...chartData.map((d) => d.value));

  // 统计卡片配置：颜色、标题、数值
  const statMetas = [
    { title: t('dashboard.totalUsers'), color: '#6366F1', value: stats ? String(stats.total_users) : '-', change: '' },
    { title: t('dashboard.activeSubscriptions'), color: '#22C55E', value: stats ? String(stats.active_subscriptions) : '-', change: '' },
    { title: t('dashboard.monthlyRevenue'), color: '#F59E0B', value: stats ? `$${stats.total_revenue.toLocaleString()}` : '-', change: '' },
    { title: t('dashboard.apiCallsToday'), color: '#EC4899', value: stats ? String(stats.api_calls_today) : '-', change: '' },
  ];

  return (
    <div style={styles.container}>
      {/* 页面头部：标题 + 时间范围选择器 */}
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
          <option value="30d">{t('dashboard.last30Days')}</option>
          <option value="7d">{t('dashboard.last7Days')}</option>
        </select>
      </header>

      {/* 统计卡片区域 */}
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

      {/* 主内容区域：收入图表 + 最近活动 */}
      <div style={styles.mainGrid}>
        {/* 收入概览图表 */}
        <div style={styles.card}>
          <div style={styles.cardHeader}>
            <h2 style={styles.cardTitle}>{t('dashboard.revenueOverview')}</h2>
            <span style={styles.cardBadge}>
              {timeRange === '30d' ? '2024' : t('dashboard.last7Days')}
            </span>
          </div>
          {/* 柱状图：根据数值自动计算高度比例 */}
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
                <span style={styles.chartLabel}>{d.label}</span>
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
            {activities.map((item, i) => (
              <div key={i} style={styles.activityItem}>
                {/* 用户头像首字母 */}
                <div style={styles.activityAvatar}>
                  {item.user.charAt(0).toUpperCase()}
                </div>
                <div style={styles.activityContent}>
                  <span style={styles.activityAction}>
                    {item.actionParams ? t(item.actionKey, item.actionParams) : t(item.actionKey)}
                  </span>
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
