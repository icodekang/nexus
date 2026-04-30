import { useState, useEffect } from 'react';
import { Check, Sparkles, Zap, TrendingUp, Activity } from 'lucide-react';
import { useI18n } from '../i18n';
import { fetchSubscription, subscribeToPlan, fetchUsage, fetchPlans, type UsageData, type PlanInfo } from '../api/client';
import { getErrorMessage } from '../utils/errors';
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

/**
 * SubscriptionPage - 订阅套餐主组件
 * @description 获取当前套餐状态，展示套餐列表，处理订阅切换
 */
export default function SubscriptionPage() {
  const { t } = useI18n();
  const [currentPlan, setCurrentPlan] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [subscribing, setSubscribing] = useState<string | null>(null);
  const [error, setError] = useState('');
  const [usage, setUsage] = useState<UsageData | null>(null);
  const [availablePlans, setAvailablePlans] = useState<PlanInfo[]>([]);

  // 加载当前订阅状态、使用量和套餐列表
  useEffect(() => {
    Promise.all([fetchSubscription(), fetchUsage(), fetchPlans()])
      .then(([sub, usageData, plansData]) => {
        if (sub.is_active && sub.subscription_plan) {
          setCurrentPlan(sub.subscription_plan);
        }
        setUsage(usageData);
        setAvailablePlans(plansData.plans);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  // 订阅/切换套餐
  const handleSubscribe = async (planKey: string) => {
    const planMap: Record<string, string> = {
      zeroToken: 'zero_token',
      monthly: 'monthly',
      autoRenew: 'monthly',
      quarterly: 'monthly',
      yearly: 'yearly',
    };
    const apiPlan = planMap[planKey] || planKey;
    setSubscribing(planKey);
    setError('');
    try {
      const res = await subscribeToPlan(apiPlan);
      setCurrentPlan(res.plan);
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setSubscribing(null);
    }
  };

  // Plan key 映射：将 API 返回的 snake_case 转换为 camelCase
  const planKeyMap: Record<string, string> = {
    'zero_token': 'zeroToken',
    'zerotoken': 'zeroToken',
    'monthly': 'monthly',
    'yearly': 'yearly',
    'team': 'team',
    'enterprise': 'enterprise',
    'none': 'none',
    'autoRenew': 'autoRenew',
    'quarterly': 'quarterly',
    'zeroToken': 'zeroToken',
  };

  // Feature 翻译映射
  const featureMap: Record<string, string> = {
    '完整 API 访问': t('subscription.featApiAccess'),
    'Full API access': t('subscription.featApiAccess'),
    '包含所有模型': t('subscription.featAllModels'),
    'All models included': t('subscription.featAllModels'),
    '邮件支持': t('subscription.featSupport'),
    'Email support': t('subscription.featSupport'),
    '优先支持': t('subscription.featPriority'),
    'Priority support': t('subscription.featPriority'),
    '使用统计分析': t('subscription.featAnalytics'),
    'Usage analytics': t('subscription.featAnalytics'),
    '自定义速率限制': t('subscription.featCustomLimits'),
    'Custom rate limits': t('subscription.featCustomLimits'),
    '浏览器模拟访问大模型': t('subscription.featBrowserAccess'),
    'Browser-based LLM access': t('subscription.featBrowserAccess'),
    '无需 API Key': t('subscription.featNoApiKey'),
    'No API key needed': t('subscription.featNoApiKey'),
    '10万 tokens/月': t('subscription.feat100kTokens'),
    '100K tokens/month': t('subscription.feat100kTokens'),
    '支持 Claude.ai': t('subscription.featClaudeSupport'),
    'Claude.ai support': t('subscription.featClaudeSupport'),
    '支持 ChatGPT': t('subscription.featChatGPTSupport'),
    'ChatGPT support': t('subscription.featChatGPTSupport'),
  };

  // 将 API PlanInfo 映射为本地 Plan 格式
  const mapApiPlanToLocal = (apiPlan: PlanInfo, index: number): Plan => {
    const period = t('subscription.perMonth');
    const billedLabel = t('subscription.billedMonthly');

    // 根据索引和 plan 名称确定 badge
    let badge: string | undefined;
    let badgeType: 'recommended' | 'best' | undefined;
    let highlighted = false;

    if (apiPlan.plan === 'autoRenew' || apiPlan.plan === 'recommended') {
      badge = t('subscription.recommended');
      badgeType = 'recommended';
      highlighted = true;
    } else if (apiPlan.plan === 'yearly' || apiPlan.plan === 'best_value') {
      badge = t('subscription.bestValue');
      badgeType = 'best';
    }

    // 翻译 features
    const translatedFeatures = apiPlan.features?.map(feat => featureMap[feat] || feat) || [];

    return {
      key: apiPlan.plan,
      price: apiPlan.plan.includes('yearly') ? `$${apiPlan.price_yearly}` : `$${apiPlan.price_monthly}`,
      period,
      badge,
      badgeType,
      billedLabel,
      features: translatedFeatures,
      highlighted,
    };
  };

  // 套餐列表配置（兜底数据）
  const defaultPlans: Plan[] = [
    {
      key: 'zeroToken',
      price: '¥10',
      period: t('subscription.perMonth'),
      badge: t('subscription.freeToken'),
      badgeType: 'recommended',
      billedLabel: t('subscription.billedMonthly'),
      features: [
        t('subscription.featBrowserAccess'),
        t('subscription.featNoApiKey'),
        t('subscription.feat100kTokens'),
        t('subscription.featClaudeSupport'),
        t('subscription.featChatGPTSupport'),
      ],
    },
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

  // 使用 API 数据或兜底硬编码 (过滤掉 enterprise)
  const displayPlans: Plan[] = (availablePlans.length > 0
    ? availablePlans.map(mapApiPlanToLocal)
    : defaultPlans).filter(plan => plan.key !== 'enterprise');

  // 获取按钮文字：当前套餐/切换/订阅
  const getButtonLabel = (planKey: string) => {
    if (subscribing === planKey) return t('login.pleaseWait');
    // Map plan keys to backend plan names for comparison
    const isCurrentPlan = currentPlan === planKey ||
      (planKey === 'zeroToken' && currentPlan === 'zero_token') ||
      (planKey === 'autoRenew' && currentPlan === 'monthly') ||
      (planKey === 'quarterly' && currentPlan === 'monthly');
    if (isCurrentPlan) return t('subscription.currentPlan');
    if (currentPlan) return t('subscription.switchPlan');
    return t('subscription.subscribe');
  };

  // 判断是否为当前套餐（考虑套餐别名映射）
  const isCurrentPlan = (planKey: string) => {
    return currentPlan === planKey ||
      (planKey === 'zeroToken' && currentPlan === 'zero_token') ||
      (planKey === 'autoRenew' && currentPlan === 'monthly') ||
      (planKey === 'quarterly' && currentPlan === 'monthly');
  };

  return (
    <div className="subscription-page">
      {/* 页面头部 */}
      <header className="subscription-header">
        <h1 className="subscription-title">{t('subscription.title')}</h1>
        <p className="subscription-subtitle">
          {loading ? t('common.loading') : t('subscription.subtitle')}
        </p>
        {error && (
          <p style={{ color: '#EF4444', fontSize: '14px', marginTop: '8px' }}>{error}</p>
        )}
      </header>

      {/* 使用量统计 */}
      {usage && (
        <div className="usage-stats">
          <div className="usage-stat">
            <Activity size={16} />
            <span className="usage-stat-label">{t('subscription.totalRequests')}</span>
            <span className="usage-stat-value">{usage.total_requests.toLocaleString()}</span>
          </div>
          <div className="usage-stat">
            <TrendingUp size={16} />
            <span className="usage-stat-label">{t('subscription.totalTokens')}</span>
            <span className="usage-stat-value">{usage.total_tokens.toLocaleString()}</span>
          </div>
          {usage.token_quota && (
            <div className="usage-stat">
              <span className="usage-stat-label">{t('subscription.quotaUsed')}</span>
              <span className="usage-stat-value">{usage.quota_used_percent.toFixed(1)}%</span>
            </div>
          )}
        </div>
      )}

      {/* 套餐卡片网格 */}
      <div className="subscription-grid">
        {displayPlans.map((plan) => (
          <div
            key={plan.key}
            className={`subscription-card ${plan.highlighted ? 'highlighted' : ''} ${isCurrentPlan(plan.key) ? 'selected' : ''}`}
          >
            {plan.badge && (
              <div className={`subscription-badge ${plan.badgeType || ''}`}>
                {plan.badgeType === 'recommended' ? <Sparkles size={12} /> : <Zap size={12} />}
                {plan.badge}
              </div>
            )}

            <div className="subscription-card-top">
              <h3 className="subscription-plan-name">
                {t(`subscription.${planKeyMap[plan.key] || plan.key}`)}
              </h3>
              <p className="subscription-plan-desc">
                {t(`subscription.${planKeyMap[plan.key] || plan.key}Desc`)}
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
              onClick={() => handleSubscribe(plan.key)}
              disabled={subscribing !== null || isCurrentPlan(plan.key)}
            >
              {getButtonLabel(plan.key)}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
