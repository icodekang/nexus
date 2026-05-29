import { useState, useEffect, useCallback } from 'react';
import { Wallet, TrendingUp, Zap, History, Shield, ArrowUpRight, DollarSign, X, CreditCard } from 'lucide-react';
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
                    <CreditCard size={16} strokeWidth={1.75} />
                    <span>{t('balance.alipay')}</span>
                  </button>
                  <button
                    className={`recharge-payment-btn ${paymentMethod === 'wechat' ? 'active' : ''}`}
                    onClick={() => setPaymentMethod('wechat')}
                  >
                    <CreditCard size={16} strokeWidth={1.75} />
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
