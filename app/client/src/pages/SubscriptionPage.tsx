import { useState } from 'react';
import { Check, Sparkles, Zap } from 'lucide-react';
import { useI18n } from '../i18n';
import './SubscriptionPage.css';

interface Plan {
  key: string;
  price: string;
  period: string;
  badge?: string;
  badgeType?: 'recommended' | 'best';
  saveLabel?: string;
  billedLabel: string;
  features: string[];
  highlighted?: boolean;
}

export default function SubscriptionPage() {
  const { t } = useI18n();
  const [selectedPlan, setSelectedPlan] = useState<string | null>(null);

  const plans: Plan[] = [
    {
      key: 'monthly',
      price: '$19',
      period: t('subscription.perMonth'),
      billedLabel: t('subscription.billedMonthly'),
      features: [
        t('subscription.featApiAccess'),
        t('subscription.featAllModels'),
        t('subscription.featSupport'),
        t('subscription.featAnalytics'),
      ],
    },
    {
      key: 'autoRenew',
      price: '$17',
      period: t('subscription.perMonth'),
      badge: t('subscription.recommended'),
      badgeType: 'recommended',
      saveLabel: t('subscription.save10'),
      billedLabel: t('subscription.billedAutoRenew'),
      highlighted: true,
      features: [
        t('subscription.featApiAccess'),
        t('subscription.featAllModels'),
        t('subscription.featSupport'),
        t('subscription.featAnalytics'),
        t('subscription.featPriority'),
      ],
    },
    {
      key: 'quarterly',
      price: '$49',
      period: t('subscription.perQuarter'),
      saveLabel: t('subscription.save14'),
      billedLabel: t('subscription.billedQuarterly'),
      features: [
        t('subscription.featApiAccess'),
        t('subscription.featAllModels'),
        t('subscription.featSupport'),
        t('subscription.featAnalytics'),
        t('subscription.featPriority'),
      ],
    },
    {
      key: 'yearly',
      price: '$199',
      period: t('subscription.perYear'),
      badge: t('subscription.bestValue'),
      badgeType: 'best',
      saveLabel: t('subscription.save17'),
      billedLabel: t('subscription.billedYearly'),
      features: [
        t('subscription.featApiAccess'),
        t('subscription.featAllModels'),
        t('subscription.featSupport'),
        t('subscription.featAnalytics'),
        t('subscription.featPriority'),
        t('subscription.featCustomLimits'),
      ],
    },
  ];

  return (
    <div className="subscription-page">
      <header className="subscription-header">
        <h1 className="subscription-title">{t('subscription.title')}</h1>
        <p className="subscription-subtitle">{t('subscription.subtitle')}</p>
      </header>

      <div className="subscription-grid">
        {plans.map((plan) => (
          <div
            key={plan.key}
            className={`subscription-card ${plan.highlighted ? 'highlighted' : ''} ${selectedPlan === plan.key ? 'selected' : ''}`}
          >
            {plan.badge && (
              <div className={`subscription-badge ${plan.badgeType || ''}`}>
                {plan.badgeType === 'recommended' ? <Sparkles size={12} /> : <Zap size={12} />}
                {plan.badge}
              </div>
            )}

            <div className="subscription-card-top">
              <h3 className="subscription-plan-name">
                {t(`subscription.${plan.key}`)}
              </h3>
              <p className="subscription-plan-desc">
                {t(`subscription.${plan.key}Desc`)}
              </p>

              <div className="subscription-price-row">
                <span className="subscription-price">{plan.price}</span>
                <span className="subscription-period">{plan.period}</span>
              </div>

              {plan.saveLabel && (
                <div className="subscription-save-tag">{plan.saveLabel}</div>
              )}

              <p className="subscription-billed">{plan.billedLabel}</p>
            </div>

            <ul className="subscription-features">
              {plan.features.map((feat, i) => (
                <li key={i} className="subscription-feature">
                  <Check size={14} className="subscription-feature-check" />
                  <span>{feat}</span>
                </li>
              ))}
            </ul>

            <button
              className={`subscription-cta ${plan.highlighted ? 'cta-highlighted' : ''}`}
              onClick={() => setSelectedPlan(plan.key)}
            >
              {selectedPlan === plan.key
                ? t('subscription.currentPlan')
                : selectedPlan
                  ? t('subscription.switchPlan')
                  : t('subscription.subscribe')}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
