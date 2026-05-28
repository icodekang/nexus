import { useState, useEffect, useCallback } from 'react';
import { Wallet, TrendingUp, Zap, Package, ChevronRight, History, Shield, ArrowUpRight, Sparkles, DollarSign } from 'lucide-react';
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
          <div className="balance-hero-badge">
            <Wallet size={14} strokeWidth={2} />
            <span>{t('balance.availableBalance')}</span>
          </div>
          <div className="balance-hero-amount">
            <span className="balance-hero-currency">$</span>
            <span className="balance-hero-value">{balNum.toFixed(6)}</span>
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
              <span className="bh-stat-value">${consumedNum.toFixed(4)}</span>
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

      {/* Packages Grid */}
      <div className="balance-section">
        <div className="balance-section-header">
          <Package size={18} strokeWidth={1.75} />
          <h2>{t('balance.buyCredits')}</h2>
        </div>
        <div className="packages-grid">
          {packages.map((pkg) => (
            <div key={pkg.id} className={`package-card ${parseFloat(pkg.bonus_credits) > 0 ? 'package-card-bonus' : ''}`}>
              <div className="package-card-top">
                <div className="package-card-name">{pkg.name}</div>
                <div className="package-card-price">
                  <span className="pc-price-dollar">$</span>
                  <span className="pc-price-value">{parseFloat(pkg.price).toFixed(0)}</span>
                </div>
              </div>
              <div className="package-card-body">
                <div className="package-card-credits">
                  <DollarSign size={18} strokeWidth={1.5} />
                  <span className="pc-credits-value">{parseFloat(pkg.credits).toFixed(2)}</span>
                  <span className="pc-credits-label">{t('balance.creditsLabel')}</span>
                </div>
                {parseFloat(pkg.bonus_credits) > 0 && (
                  <div className="package-card-bonus-tag">
                    <Sparkles size={12} />
                    <span>+${parseFloat(pkg.bonus_credits).toFixed(2)} {t('balance.bonusLabel')}</span>
                  </div>
                )}
              </div>
              <button
                className="package-card-btn"
                onClick={() => handlePurchase(pkg.id)}
                disabled={purchasing === pkg.id}
              >
                {purchasing === pkg.id ? (
                  <span className="spinner" />
                ) : (
                  <>
                    <span>{t('balance.purchase')}</span>
                    <ChevronRight size={14} strokeWidth={2} />
                  </>
                )}
              </button>
            </div>
          ))}
        </div>
      </div>

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
                  {charge.is_free ? t('balance.freeZero') : `-$${parseFloat(charge.total_cost).toFixed(6)}`}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
