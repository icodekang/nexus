import { useState, useEffect, useCallback } from 'react';
import { Wallet, TrendingUp, Zap, History, Shield, ArrowUpRight, DollarSign, X } from 'lucide-react';
import { useAuthStore } from '../stores/authStore';
import { useI18n } from '../i18n';
import { fetchBalance, fetchUsage, fetchCharges, fetchPackages, purchasePackage, type BalanceData, type UsageData, type ChargeItem, type TokenPackage } from '../api/client';
import { getErrorMessage } from '../utils/errors';
import './BalancePage.css';

export default function BalancePage() {
  const { t, locale } = useI18n();
  const { isAuthenticated } = useAuthStore();
  const [balance, setBalance] = useState<BalanceData | null>(null);
  const [usage, setUsage] = useState<UsageData | null>(null);
  const [charges, setCharges] = useState<ChargeItem[]>([]);
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
    // Use first available package for purchase, amount could be passed as param
    if (packages.length === 0) return;
    setPurchasing(packages[0].id);
    setError('');
    try {
      const res = await purchasePackage(packages[0].id);
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
      const [bal, usg, chg, pkgs] = await Promise.all([
        fetchBalance(), fetchUsage(), fetchCharges(1, 10), fetchPackages()
      ]);
      setBalance(bal);
      setUsage(usg);
      setCharges(chg.data);
      setPackages(pkgs.packages);
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated]);

  useEffect(() => { loadData(); }, [loadData]);

  const handlePurchase = async (pkgId: string) => {
    setPurchasing(pkgId);
    setError('');
    try {
      const res = await purchasePackage(pkgId);
      setBalance(prev => prev ? { ...prev, balance: res.balance } : prev);
      await loadData();
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setPurchasing(null);
    }
  };

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
  const consumedNum = balance ? parseFloat(balance.total_consumed) : 0;
  const currency = locale === 'zh' ? '¥' : '$';

  return (
    <div className="balance-page">
      {/* Header */}
      <div className="balance-header">
        <div className="balance-header-left">
          <h1 className="balance-title">{t('balance.title')}</h1>
          <p className="balance-subtitle">{t('balance.subtitle')}</p>
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
            <button
              className="balance-recharge-btn"
              onClick={() => setShowRecharge(true)}
            >
              <DollarSign size={12} strokeWidth={2} />
              <span>{t('balance.recharge')}</span>
            </button>
          </div>
          <div className="balance-hero-meta">
            <div className="balance-hero-stat">
              <span className="bh-stat-value">{usage ? usage.total_tokens.toLocaleString() : 0}</span>
              <span className="bh-stat-label">{t('balance.tokensUsed30d')}</span>
            </div>
            <div className="balance-hero-divider" />
            <div className="balance-hero-stat">
              <span className="bh-stat-value">{usage ? usage.total_requests.toLocaleString() : 0}</span>
              <span className="bh-stat-label">{t('balance.requests30d')}</span>
            </div>
            <div className="balance-hero-divider" />
            <div className="balance-hero-stat">
              <span className="bh-stat-value">{currency}{consumedNum.toFixed(4)}</span>
              <span className="bh-stat-label">{t('balance.spent30d')}</span>
            </div>
          </div>
        </div>
      </div>

      {error && (
        <div className="balance-error">
          <span>{error}</span>
        </div>
      )}

      {/* Charge History */}
      <div className="balance-section">
        <div className="balance-section-header">
          <History size={18} strokeWidth={1.75} />
          <h2>{t('balance.recentCharges')}</h2>
        </div>
        <div className="charges-list">
          {charges.length === 0 ? (
            <div className="charges-empty">
              <Zap size={24} strokeWidth={1.5} />
              <span>{t('balance.noCharges')}</span>
            </div>
          ) : (
            charges.map((charge) => (
              <div key={charge.id} className="charge-row">
                <div className="charge-row-left">
                  <div className={`charge-row-icon ${charge.is_free ? 'icon-free' : ''}`}>
                    {charge.is_free ? <Shield size={14} /> : <ArrowUpRight size={14} />}
                  </div>
                  <div className="charge-row-info">
                    <span className="charge-row-model">{charge.model}</span>
                    <span className="charge-row-meta">
                      {charge.input_tokens + charge.output_tokens} {t('balance.tokensLabel')}
                      {charge.is_free ? ` · ${t('balance.free')}` : ''}
                    </span>
                  </div>
                </div>
                <div className={`charge-row-cost ${charge.is_free ? 'cost-free' : ''}`}>
                  {charge.is_free ? t('balance.freeZero') : `-${currency}${parseFloat(charge.total_cost).toFixed(6)}`}
                </div>
              </div>
            ))
          )}
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
                  <input
                    type="number"
                    className="recharge-custom-input"
                    placeholder="0.00"
                    value={customAmount}
                    onChange={e => { setCustomAmount(e.target.value); setRechargeAmount(null); }}
                    min="0"
                    step="0.01"
                  />
                </div>
              </div>

              <div className="recharge-payment">
                <span className="recharge-payment-label">{t('balance.paymentMethod')}</span>
                <div className="recharge-payment-options">
                  <button
                    className={`recharge-payment-btn ${paymentMethod === 'alipay' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('alipay')}
                  >
                    <svg className="payment-icon-svg alipay-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <path d="M19.695 15.07c3.426 1.158 4.203 1.22 4.203 1.22V3.846c0-2.124-1.705-3.845-3.81-3.845H3.914C1.808.001.102 1.722.102 3.846v16.31c0 2.123 1.706 3.845 3.813 3.845h16.173c2.105 0 3.81-1.722 3.81-3.845v-.157s-6.19-2.602-9.315-4.119c-2.096 2.602-4.8 4.181-7.607 4.181-4.75 0-6.361-4.19-4.112-6.949.49-.602 1.324-1.175 2.617-1.497 2.025-.502 5.247.313 8.266 1.317a16.796 16.796 0 0 0 1.341-3.302H5.781v-.952h4.799V6.975H4.77v-.953h5.81V3.591s0-.409.411-.409h2.347v2.84h5.744v.951h-5.744v1.704h4.69a19.453 19.453 0 0 1-1.986 5.06c1.424.52 2.702 1.011 3.654 1.333m-13.81-2.032c-.596.06-1.71.325-2.321.869-1.83 1.608-.735 4.55 2.968 4.55 2.151 0 4.301-1.388 5.99-3.61-2.403-1.182-4.438-2.028-6.637-1.809" fill="currentColor" />
                    </svg>
                    <span>{t('balance.alipay')}</span>
                  </button>
                  <button
                    className={`recharge-payment-btn ${paymentMethod === 'wechat' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('wechat')}
                  >
                    <svg className="payment-icon-svg wechat-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
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
                <span className="recharge-total-amount">
                  ${selectedAmount ? selectedAmount.toFixed(2) : '0.00'}
                </span>
              </div>
              <button
                className="recharge-confirm-btn"
                onClick={handleRechargeConfirm}
                disabled={!selectedAmount || selectedAmount <= 0 || purchasing !== null}
              >
                {purchasing !== null ? (
                  <span className="spinner" />
                ) : (
                  t('balance.confirmRecharge')
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
