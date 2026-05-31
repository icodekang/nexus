import { useState, useEffect, useCallback } from 'react';
import { Wallet, X, BarChart3, Receipt, Clock } from 'lucide-react';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import { fetchBalance, fetchDailyUsage, fetchDailyModelUsage, fetchTransactions, fetchPackages, recharge, type BalanceData, type DailyUsageItem, type DailyModelUsage, type TransactionItem, type TokenPackage } from '../api/client';
import { getErrorMessage } from '../utils/errors';
import './BalancePage.css';

export default function BalancePage() {
  const { t, locale } = useI18n();
  const { isAuthenticated } = useAuthStore();
  const [balance, setBalance] = useState<BalanceData | null>(null);
  const [dailyUsage, setDailyUsage] = useState<DailyUsageItem[]>([]);
  const [modelUsage, setModelUsage] = useState<DailyModelUsage[]>([]);
  const [transactions, setTransactions] = useState<TransactionItem[]>([]);
  const [packages, setPackages] = useState<TokenPackage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [purchasing, setPurchasing] = useState<string | null>(null);

  const [showRecharge, setShowRecharge] = useState(false);
  const [rechargeAmount, setRechargeAmount] = useState<number | null>(null);
  const [customAmount, setCustomAmount] = useState('');
  const [paymentMethod, setPaymentMethod] = useState<'alipay' | 'wechat'>('alipay');

  const PRESET_AMOUNTS = [10, 20, 50, 100, 200, 500];
  const selectedAmount = rechargeAmount ?? (customAmount ? parseFloat(customAmount) : null);

  const handleRechargeConfirm = async () => {
    if (!selectedAmount || selectedAmount <= 0) return;
    setPurchasing('direct');
    setError('');
    try {
      const res = await recharge(selectedAmount);
      setBalance(prev => prev ? { ...prev, balance: res.balance } : prev);
      await loadData();
      setShowRecharge(false);
      setRechargeAmount(null);
      setCustomAmount('');
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setPurchasing(null);
    }
  };

  const loadData = useCallback(async () => {
    if (!isAuthenticated) { setLoading(false); return; }
    try {
      const [bal, daily, model, txs, pkgs] = await Promise.all([
        fetchBalance(), fetchDailyUsage(), fetchDailyModelUsage(), fetchTransactions(), fetchPackages()
      ]);
      setBalance(bal);
      setDailyUsage(daily.data);
      setModelUsage(model.data);
      setTransactions(txs.data);
      setPackages(pkgs.packages);
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated]);

  useEffect(() => { loadData(); }, [loadData]);

  if (loading) {
    return (
      <div className="balance-page">
        <div className="balance-skeleton">
          <div className="sk-card pulse" />
          <div className="sk-card pulse" />
        </div>
      </div>
    );
  }

  const balNum = balance ? parseFloat(balance.balance) : 0;
  const currency = '¥';
  const totalTokens = dailyUsage.reduce((s, d) => s + d.input_tokens + d.output_tokens, 0);
  const totalCost = dailyUsage.reduce((s, d) => s + parseFloat(d.total_cost || '0'), 0);

  const formatDate = (iso: string, loc: string) => {
    const d = new Date(iso);
    if (loc === 'zh') {
      return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
    }
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="balance-page">
      {/* Header */}
      <div className="balance-header">
        <div className="balance-header-left">
          <h1 className="balance-title">{t('balance.title')}</h1>
        </div>
      </div>

      {/* Balance Hero Card */}
      <div className="balance-hero">
        <div className="balance-hero-glow" />
        <div className="balance-hero-content">
          <div className="balance-hero-top">
            <div className="balance-hero-left">
              <div className="balance-hero-badge">
                <Wallet size={14} strokeWidth={2} />
                <span>{t('balance.availableBalance')}</span>
              </div>
              <div className="balance-hero-amount">
                <span className="balance-hero-currency">{currency}</span>
                <span className="balance-hero-value">{balNum.toFixed(6)}</span>
              </div>
            </div>
            <button className="balance-recharge-btn" onClick={() => setShowRecharge(true)}>
              <span className="balance-recharge-icon">{currency}</span>
              <span>{t('balance.recharge')}</span>
            </button>
          </div>
          <div className="balance-hero-meta">
            <div className="balance-hero-stat">
              <span className="bh-stat-value">{totalTokens.toLocaleString()}</span>
              <span className="bh-stat-label">{t('balance.tokens7d')}</span>
            </div>
            <div className="balance-hero-divider" />
            <div className="balance-hero-stat">
              <span className="bh-stat-value">{dailyUsage.length}</span>
              <span className="bh-stat-label">{t('balance.daysLabel')}</span>
            </div>
            <div className="balance-hero-divider" />
            <div className="balance-hero-stat">
              <span className="bh-stat-value">{currency}{totalCost.toFixed(4)}</span>
              <span className="bh-stat-label">{t('balance.spent7d')}</span>
            </div>
          </div>
        </div>
      </div>

      {error && (
        <div className="balance-error">
          <span>{error}</span>
        </div>
      )}

      {/* Usage Chart */}
      <div className="balance-section">
        <div className="balance-section-header">
          <BarChart3 size={18} strokeWidth={1.75} />
          <h2>{t('balance.usageChart')}</h2>
        </div>
        <div className="usage-chart-container">
          <UsageChart data={dailyUsage} modelData={modelUsage} currency={currency} />
        </div>
      </div>

      {/* Orders */}
      <div className="balance-section">
        <div className="balance-section-header">
          <Receipt size={18} strokeWidth={1.75} />
          <h2>{t('balance.orders')}</h2>
        </div>
        <div className="orders-list-container">
          <div className="orders-list">
            {transactions.length === 0 ? (
              <div className="charges-empty">
                <Clock size={24} strokeWidth={1.5} />
                <span>{t('balance.noOrders')}</span>
              </div>
            ) : (
              transactions.filter(t => t.type === 'token_purchase').slice(0, 7).map((tx) => (
                <div key={tx.id} className="order-row">
                  <div className="order-row-left">
                    <div className="order-icon">
                      <Receipt size={14} />
                    </div>
                    <div className="order-info">
                      <span className="order-id" title={tx.id}>{tx.id.slice(0, 8)}...</span>
                      <span className="order-time">{formatDate(tx.created_at, locale)}</span>
                    </div>
                  </div>
                  <div className="order-row-right">
                    <span className="order-plan">{tx.plan || t('balance.recharge')}</span>
                    <span className="order-amount">+{currency}{tx.amount.toFixed(2)}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Recharge Modal */}
      {showRecharge && (
        <div className="recharge-overlay" onClick={() => setShowRecharge(false)}>
          <div className="recharge-modal" onClick={e => e.stopPropagation()}>
            <div className="recharge-modal-header">
              <h2>{t('balance.rechargeTitle')}</h2>
              <button className="recharge-modal-close" onClick={() => setShowRecharge(false)}>
                <X size={18} strokeWidth={2} />
              </button>
            </div>
            <div className="recharge-modal-body">
              <div className="recharge-amounts">
                {PRESET_AMOUNTS.map(amt => (
                  <button
                    key={amt}
                    className={`recharge-amount-btn ${rechargeAmount === amt ? 'active' : ''}`}
                    onClick={() => { setRechargeAmount(amt); setCustomAmount(''); }}
                  >
                    <span className="recharge-amount-currency">{currency}</span>
                    <span className="recharge-amount-value">{amt}</span>
                  </button>
                ))}
              </div>
              <div className="recharge-custom">
                <span className="recharge-custom-label">{t('balance.customAmount')}</span>
                <div className="recharge-custom-input-wrap">
                  <span className="recharge-custom-prefix">{currency}</span>
                  <input type="number" className="recharge-custom-input" placeholder="0.00"
                    value={customAmount} onChange={e => { setCustomAmount(e.target.value); setRechargeAmount(null); }}
                    min="0" step="0.01" />
                </div>
              </div>
              <div className="recharge-payment">
                <span className="recharge-payment-label">{t('balance.paymentMethod')}</span>
                <div className="recharge-payment-options">
                  <button className={`recharge-payment-btn ${paymentMethod === 'alipay' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('alipay')}>
                    <svg className="payment-icon-svg alipay-icon" width="18" height="18" viewBox="0 0 24 24" fill="none">
                      <path d="M19.695 15.07c3.426 1.158 4.203 1.22 4.203 1.22V3.846c0-2.124-1.705-3.845-3.81-3.845H3.914C1.808.001.102 1.722.102 3.846v16.31c0 2.123 1.706 3.845 3.813 3.845h16.173c2.105 0 3.81-1.722 3.81-3.845v-.157s-6.19-2.602-9.315-4.119c-2.096 2.602-4.8 4.181-7.607 4.181-4.75 0-6.361-4.19-4.112-6.949.49-.602 1.324-1.175 2.617-1.497 2.025-.502 5.247.313 8.266 1.317a16.796 16.796 0 0 0 1.341-3.302H5.781v-.952h4.799V6.975H4.77v-.953h5.81V3.591s0-.409.411-.409h2.347v2.84h5.744v.951h-5.744v1.704h4.69a19.453 19.453 0 0 1-1.986 5.06c1.424.52 2.702 1.011 3.654 1.333m-13.81-2.032c-.596.06-1.71.325-2.321.869-1.83 1.608-.735 4.55 2.968 4.55 2.151 0 4.301-1.388 5.99-3.61-2.403-1.182-4.438-2.028-6.637-1.809" fill="currentColor" />
                    </svg>
                    <span>{t('balance.alipay')}</span>
                  </button>
                  <button className={`recharge-payment-btn ${paymentMethod === 'wechat' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('wechat')}>
                    <svg className="payment-icon-svg wechat-icon" width="18" height="18" viewBox="0 0 24 24" fill="none">
                      <path d="M8.691 2.188C3.891 2.188 0 5.476 0 9.53c0 2.212 1.17 4.203 3.002 5.55a.59.59 0 0 1 .213.665l-.39 1.48c-.019.07-.048.141-.048.213 0 .163.13.295.29.295a.326.326 0 0 0 .167-.054l1.903-1.114a.864.864 0 0 1 .717-.098 10.16 10.16 0 0 0 2.837.403c.276 0 .543-.027.811-.05-.857-2.578.157-4.972 1.932-6.446 1.703-1.415 3.882-1.98 5.853-1.838-.576-3.583-4.196-6.348-8.596-6.348zM5.785 5.991c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 0 1-1.162 1.178A1.17 1.17 0 0 1 4.623 7.17c0-.651.52-1.18 1.162-1.18zm5.813 0c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 0 1-1.162 1.178 1.17 1.17 0 0 1-1.162-1.178c0-.651.52-1.18 1.162-1.18zm5.34 2.867c-1.797-.052-3.746.512-5.28 1.786-1.72 1.428-2.687 3.72-1.78 6.22.942 2.453 3.666 4.229 6.884 4.229.826 0 1.622-.12 2.361-.336a.722.722 0 0 1 .598.082l1.584.926a.272.272 0 0 0 .14.047c.134 0 .24-.111.24-.247 0-.06-.023-.12-.038-.177l-.327-1.233a.582.582 0 0 1-.023-.156.49.49 0 0 1 .201-.398C23.024 18.48 24 16.82 24 14.98c0-3.21-2.931-5.837-6.656-6.088V8.89c-.135-.01-.27-.027-.407-.03zm-2.53 3.274c.535 0 .969.44.969.982a.976.976 0 0 1-.969.983.976.976 0 0 1-.969-.983c0-.542.434-.982.97-.982zm4.844 0c.535 0 .969.44.969.982a.976.976 0 0 1-.969.983.976.976 0 0 1-.969-.983c0-.542.434-.982.969-.982z" fill="currentColor" />
                    </svg>
                    <span>{t('balance.wechat')}</span>
                  </button>
                </div>
              </div>
            </div>
            <div className="recharge-modal-footer">
              <div className="recharge-total">
                <span className="recharge-total-label">{t('balance.total')}</span>
                <span className="recharge-total-amount">{currency}{selectedAmount ? selectedAmount.toFixed(2) : '0.00'}</span>
              </div>
              <button className="recharge-confirm-btn" onClick={handleRechargeConfirm}
                disabled={!selectedAmount || selectedAmount <= 0 || purchasing !== null}>
                {purchasing !== null ? <span className="spinner" /> : t('balance.confirmRecharge')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function UsageChart({ data, modelData, currency }: { data: DailyUsageItem[]; modelData: DailyModelUsage[]; currency: string }) {
  const [hovered, setHovered] = useState<number | null>(null);

  if (data.length === 0) {
    return (
      <div className="charges-empty">
        <BarChart3 size={24} strokeWidth={1.5} />
        <span>No usage data yet</span>
      </div>
    );
  }

  const pad = { top: 24, right: 24, bottom: 32, left: 48 };
  const width = 640;
  const height = 280;
  const chartW = width - pad.left - pad.right;
  const chartH = height - pad.top - pad.bottom;

  const totals = data.map(d => d.input_tokens + d.output_tokens);
  const costs = data.map(d => parseFloat(d.total_cost || '0'));
  const maxTokens = Math.max(...totals, 1);
  const maxTokensNice = Math.pow(10, Math.ceil(Math.log10(maxTokens)));
  const barW = Math.max(14, Math.min(44, chartW / data.length - 14));
  const gap = (chartW - barW * data.length) / (data.length + 1);

  const yTicks = 4;
  const gridLines: number[] = [];
  for (let i = 0; i <= yTicks; i++) {
    gridLines.push(Math.round(maxTokensNice * i / yTicks));
  }

  function yPos(val: number) { return pad.top + chartH - (val / maxTokensNice) * chartH; }

  function formatTokens(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(0)}K`;
    return String(n);
  }

  const days = data.map(d => {
    const parts = d.day.split('-');
    return `${parts[1]}/${parts[2]}`;
  });

  return (
    <div className="usage-chart-wrapper">
      <div className="usage-chart-container-inner">
        <svg viewBox={`0 0 ${width} ${height}`} className="usage-chart-svg"
          onMouseLeave={() => setHovered(null)}>
          {gridLines.map((val, i) => (
            <g key={`grid-${i}`}>
              <line x1={pad.left} y1={yPos(val)} x2={width - pad.right} y2={yPos(val)}
                stroke="#F0F0F0" strokeWidth="0.5" />
              <text x={pad.left - 8} y={yPos(val) + 4} textAnchor="end"
                fill="#A1A1AA" fontSize="10" fontFamily="DM Sans, sans-serif">
                {formatTokens(val)}
              </text>
            </g>
          ))}

          {data.map((d, i) => {
            const total = totals[i];
            const x = pad.left + gap + i * (barW + gap);
            const h = (total / maxTokensNice) * chartH || 2;
            const isHovered = hovered === i;

            return (
              <g key={`bar-${i}`}>
                <rect x={x} y={yPos(total)} width={barW} height={h}
                  rx="4" fill={isHovered ? '#6366F1' : '#818CF8'}
                  opacity={isHovered ? 1 : 0.72}
                  onMouseEnter={() => setHovered(i)}
                  style={{ cursor: 'pointer' }}
                />
              </g>
            );
          })}

          {days.map((label, i) => {
            const x = pad.left + gap + i * (barW + gap) + barW / 2;
            return (
              <text key={`xlabel-${i}`} x={x} y={height - 6} textAnchor="middle"
                fill="#A1A1AA" fontSize="10" fontFamily="DM Sans, sans-serif">
                {label}
              </text>
            );
          })}
        </svg>

        {hovered !== null && modelData[hovered] && (
          <div className="chart-tooltip-popup">
            {modelData[hovered].models.length > 0 ? (
              <>
                {modelData[hovered].models.map((m, j) => (
                  <div key={j} className="chart-tooltip-model-row">
                    <span className="chart-tooltip-model-name">{m.model}</span>
                    <span className="chart-tooltip-model-cost">{currency}{parseFloat(m.cost).toFixed(6)}</span>
                  </div>
                ))}
                <div className="chart-tooltip-divider" />
              </>
            ) : null}
            <div className="chart-tooltip-total-row">
              <span className="chart-tooltip-total-label">Total</span>
              <span className="chart-tooltip-total-cost">{currency}{parseFloat(modelData[hovered].total_cost).toFixed(6)}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
