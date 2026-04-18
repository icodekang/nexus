/**
 * @file Transactions - 交易记录管理页面
 * 展示用户订阅和支付交易记录，支持按类型和状态筛选
 */
import { useState, useEffect, useCallback, useMemo } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import { fetchTransactions, type AdminTransaction } from '../api/admin';

// 订阅套餐颜色映射
const planColors: Record<string, string> = {
  yearly: '#6366F1',
  monthly: '#3B82F6',
  team: '#F59E0B',
  enterprise: '#EC4899',
};

/**
 * Transactions - 交易记录主组件
 * @description 获取交易列表，支持类型/状态筛选，展示统计摘要
 */
export default function Transactions() {
  const { t } = useI18n();
  const [transactions, setTransactions] = useState<AdminTransaction[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [perPage] = useState(20);
  const [loading, setLoading] = useState(true);
  // 筛选状态
  const [typeFilter, setTypeFilter] = useState('');
  const [statusFilter, setStatusFilter] = useState('');
  const [detailTx, setDetailTx] = useState<AdminTransaction | null>(null);

  const totalPages = Math.ceil(total / perPage);

  // 加载交易数据
  const loadTransactions = useCallback(() => {
    setLoading(true);
    fetchTransactions(page, perPage, typeFilter, statusFilter)
      .then((res) => {
        setTransactions(res.data);
        setTotal(res.total);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [page, perPage, typeFilter, statusFilter]);

  useEffect(() => {
    loadTransactions();
  }, [loadTransactions]);

  // 筛选条件变化时重置页码
  useEffect(() => {
    setPage(1);
  }, [typeFilter, statusFilter]);

  // 交易类型和状态选项
  const txTypes = ['purchase', 'refund', 'renewal'];
  const txStatuses = ['completed', 'refunded', 'pending'];

  // 获取状态对应的本地化标签
  const statusLabel = (s: string) => {
    if (s === 'completed') return t('transactions.completed');
    if (s === 'refunded') return t('transactions.refunded');
    if (s === 'pending') return t('transactions.pending');
    return s;
  };

  // 获取交易类型对应的本地化标签
  const typeLabel = (s: string) => {
    if (s === 'purchase') return t('transactions.purchase');
    if (s === 'refund') return t('transactions.refund');
    if (s === 'renewal') return t('transactions.renewal');
    return s;
  };

  // 获取状态对应的颜色
  const statusColor = (s: string) => {
    if (s === 'completed') return '#22C55E';
    if (s === 'refunded') return '#F59E0B';
    return '#A1A1AA';
  };

  // 计算今日收入和平均订单金额（基于当前页数据，仅供参考）
  const summaryData = useMemo(() => {
    const revenueToday = transactions
      .filter((tx) => tx.status === 'completed' && tx.created_at.slice(0, 10) === new Date().toISOString().slice(0, 10))
      .reduce((sum, tx) => sum + tx.amount, 0);
    const avgOrder = transactions.length > 0
      ? transactions.filter((tx) => tx.amount > 0).reduce((sum, tx) => sum + tx.amount, 0) / Math.max(1, transactions.filter((tx) => tx.amount > 0).length)
      : 0;
    return {
      revenueToday: `$${revenueToday.toFixed(2)}`,
      count: total,
      avgOrder: `$${avgOrder.toFixed(2)}`,
    };
  }, [transactions, total]);

  return (
    <div style={styles.container}>
      {/* 页面头部：标题 + 筛选器 */}
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('transactions.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? 'Loading...' : t('transactions.subtitle')}
          </p>
        </div>
        {/* 交易类型和状态筛选 */}
        <div style={styles.filters}>
          <select
            style={styles.select}
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value)}
          >
            <option value="">{t('transactions.allTypes')}</option>
            {txTypes.map((tp) => (
              <option key={tp} value={tp}>{typeLabel(tp)}</option>
            ))}
          </select>
          <select
            style={styles.select}
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
          >
            <option value="">{t('transactions.allStatus')}</option>
            {txStatuses.map((s) => (
              <option key={s} value={s}>{statusLabel(s)}</option>
            ))}
          </select>
        </div>
      </header>

      {/* 统计摘要卡片 */}
      <div style={styles.summaryGrid}>
        {[
          { label: t('transactions.revenueToday'), value: summaryData.revenueToday, color: '#6366F1' },
          { label: t('transactions.transactionsCount'), value: String(total), color: '#22C55E' },
          { label: t('transactions.avgOrder'), value: summaryData.avgOrder, color: '#F59E0B' },
        ].map((s, i) => (
          <div key={i} style={styles.summaryCard}>
            <div style={{ ...styles.summaryDot, backgroundColor: s.color }} />
            <span style={styles.summaryLabel}>{s.label}</span>
            <span style={styles.summaryValue}>{loading ? '-' : s.value}</span>
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
              <th style={styles.th}>{t('transactions.thDate')}</th>
              <th style={{ ...styles.th, paddingRight: '20px', textAlign: 'right' }}></th>
            </tr>
          </thead>
          <tbody>
            {transactions.map((tx) => (
              <tr key={tx.id} style={styles.tr}>
                <td style={{ ...styles.td, paddingLeft: '20px' }}>
                  <div style={styles.userCell}>
                    <div style={styles.userAvatar}>{tx.user_email.charAt(0).toUpperCase()}</div>
                    <span style={styles.email}>{tx.user_email}</span>
                  </div>
                </td>
                <td style={styles.td}>
                  <span style={styles.type}>{typeLabel(tx.transaction_type)}</span>
                </td>
                <td style={styles.td}>
                  {tx.plan ? (
                    <span style={{
                      ...styles.planBadge,
                      color: planColors[tx.plan] || '#A1A1AA',
                      backgroundColor: `${planColors[tx.plan] || '#A1A1AA'}12`,
                    }}>
                      {tx.plan}
                    </span>
                  ) : (
                    <span style={{ color: '#A1A1AA' }}>-</span>
                  )}
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.amount,
                    color: tx.amount < 0 ? '#EF4444' : '#18181B',
                  }}>
                    {tx.amount < 0 ? `-$${Math.abs(tx.amount)}` : `$${tx.amount}`}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.status,
                    color: statusColor(tx.status),
                  }}>
                    <span style={{
                      ...styles.statusDot,
                      backgroundColor: statusColor(tx.status),
                    }} />
                    {statusLabel(tx.status)}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={styles.date}>{tx.created_at.slice(0, 10)}</span>
                </td>
                <td style={{ ...styles.td, paddingRight: '20px', textAlign: 'right' }}>
                  <button style={styles.actionBtn} onClick={() => setDetailTx(tx)}>
                    {t('common.viewDetails')}
                  </button>
                </td>
              </tr>
            ))}
            {!loading && transactions.length === 0 && (
              <tr>
                <td colSpan={7} style={{ ...styles.td, textAlign: 'center', color: '#A1A1AA', padding: '40px' }}>
                  No transactions found
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* 分页控件 */}
      {totalPages > 1 && (
        <div style={styles.pagination}>
          <span style={styles.paginationInfo}>
            {t('transactions.pageInfo', { page, totalPages, total })}
          </span>
          <div style={styles.paginationButtons}>
            <button
              style={{ ...styles.pageBtn, opacity: page <= 1 ? 0.5 : 1 }}
              disabled={page <= 1}
              onClick={() => setPage((p) => Math.max(1, p - 1))}
            >
              {t('common.prev')}
            </button>
            <button
              style={{ ...styles.pageBtn, opacity: page >= totalPages ? 0.5 : 1 }}
              disabled={page >= totalPages}
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            >
              {t('common.next')}
            </button>
          </div>
        </div>
      )}

      <Modal open={!!detailTx} onClose={() => setDetailTx(null)} title={t('transactions.detailTitle')}>
        {detailTx && (
          <div style={detailStyles.grid}>
            <DetailRow label={t('transactions.detailId')} value={detailTx.id} />
            <DetailRow label={t('transactions.detailUser')} value={detailTx.user_email} />
            <DetailRow label={t('transactions.detailType')} value={typeLabel(detailTx.transaction_type)} />
            <DetailRow label={t('transactions.detailPlan')} value={detailTx.plan || '-'} />
            <DetailRow label={t('transactions.detailAmount')} value={detailTx.amount < 0 ? `-$${Math.abs(detailTx.amount)}` : `$${detailTx.amount}`} />
            <DetailRow label={t('transactions.detailStatus')} value={statusLabel(detailTx.status)} />
            <DetailRow label={t('transactions.detailDate')} value={detailTx.created_at.slice(0, 10)} />
            {detailTx.description && (
              <DetailRow label="Description" value={detailTx.description} />
            )}
            <div style={detailStyles.actions}>
              <button style={detailStyles.closeBtn} onClick={() => setDetailTx(null)}>
                {t('common.close')}
              </button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={detailStyles.row}>
      <span style={detailStyles.label}>{label}</span>
      <span style={detailStyles.value}>{value}</span>
    </div>
  );
}

const detailStyles: Record<string, React.CSSProperties> = {
  grid: { display: 'flex', flexDirection: 'column', gap: '12px' },
  row: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '8px 0',
    borderBottom: '1px solid #F5F5F4',
  },
  label: {
    fontSize: '12px',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
  value: {
    fontSize: '13px',
    fontWeight: '500',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  actions: {
    display: 'flex',
    justifyContent: 'flex-end',
    marginTop: '8px',
  },
  closeBtn: {
    padding: '8px 16px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF',
    fontSize: '12px',
    fontWeight: '500',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
};

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
  actions: { display: 'flex', gap: '8px', justifyContent: 'flex-end' },
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
  pagination: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: '16px',
    padding: '0 4px',
  },
  paginationInfo: {
    fontSize: '13px',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
  paginationButtons: {
    display: 'flex',
    gap: '8px',
  },
  pageBtn: {
    padding: '6px 14px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF',
    fontSize: '12px',
    fontWeight: '500',
    color: '#18181B',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
};
