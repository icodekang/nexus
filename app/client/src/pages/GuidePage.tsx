import { BookOpen, Key, MessageSquare, Layers, Zap, ArrowRight, Code2, Send } from 'lucide-react';
import { useI18n } from '../i18n';
import './GuidePage.css';

export default function GuidePage() {
  const { t } = useI18n();

  const steps = [
    {
      icon: Key,
      title: t('guide.step1Title'),
      desc: t('guide.step1Desc'),
      code: `# Your key will look like:\nsk-nexus-a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4`,
    },
    {
      icon: Layers,
      title: t('guide.step2Title'),
      desc: t('guide.step2Desc'),
      code: null,
    },
    {
      icon: MessageSquare,
      title: t('guide.step3Title'),
      desc: t('guide.step3Desc'),
      code: null,
    },
    {
      icon: Code2,
      title: t('guide.step4Title'),
      desc: t('guide.step4Desc'),
      code: `curl https://your-nexus-domain.com/v1/chat/completions \\
  -H "Authorization: Bearer sk-nexus-your-key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'`,
    },
  ];

  const endpoints = [
    { method: 'POST', path: '/v1/chat/completions', desc: t('guide.epChat') },
    { method: 'GET', path: '/v1/models', desc: t('guide.epModels') },
    { method: 'POST', path: '/v1/embeddings', desc: t('guide.epEmbeddings') },
    { method: 'GET', path: '/v1/me/usage', desc: t('guide.epUsage') },
    { method: 'GET', path: '/v1/me/keys', desc: t('guide.epKeys') },
  ];

  return (
    <div className="guide-page">
      <header className="guide-header">
        <h1 className="guide-title">{t('guide.title')}</h1>
        <p className="guide-subtitle">{t('guide.subtitle')}</p>
      </header>

      {/* Steps */}
      <div className="guide-steps">
        {steps.map((step, i) => (
          <div key={i} className="guide-step">
            <div className="guide-step-number">{i + 1}</div>
            <div className="guide-step-content">
              <div className="guide-step-header">
                <step.icon size={18} />
                <h3 className="guide-step-title">{step.title}</h3>
              </div>
              <p className="guide-step-desc">{step.desc}</p>
              {step.code && (
                <pre className="guide-code">
                  <code>{step.code}</code>
                </pre>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* API Reference */}
      <section className="guide-section">
        <h2 className="guide-section-title">
          <Zap size={18} />
          {t('guide.apiEndpoints')}
        </h2>
        <div className="guide-endpoints">
          {endpoints.map((ep, i) => (
            <div key={i} className="guide-endpoint">
              <span className={`guide-method ${ep.method.toLowerCase()}`}>{ep.method}</span>
              <code className="guide-path">{ep.path}</code>
              <span className="guide-endpoint-desc">{ep.desc}</span>
            </div>
          ))}
        </div>
      </section>

      {/* Streaming */}
      <section className="guide-section">
        <h2 className="guide-section-title">
          <Send size={18} />
          {t('guide.streaming')}
        </h2>
        <p className="guide-section-desc">
          {t('guide.streamingDesc')}
        </p>
        <pre className="guide-code">
          <code>{`curl https://your-nexus-domain.com/v1/chat/completions \\
  -H "Authorization: Bearer sk-nexus-your-key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Tell me a story"}],
    "stream": true
  }'`}</code>
        </pre>
      </section>

      {/* Subscription info */}
      <section className="guide-section">
        <h2 className="guide-section-title">
          <BookOpen size={18} />
          {t('guide.subscriptionPlans')}
        </h2>
        <div className="guide-plans">
          {[
            { name: t('guide.free'), quota: t('guide.freeQuota'), rate: t('guide.freeRate'), desc: t('guide.freeDesc') },
            { name: t('guide.monthly'), quota: t('guide.monthlyQuota'), rate: t('guide.monthlyRate'), desc: t('guide.monthlyDesc') },
            { name: t('guide.yearly'), quota: t('guide.yearlyQuota'), rate: t('guide.yearlyRate'), desc: t('guide.yearlyDesc') },
            { name: t('guide.team'), quota: t('guide.teamQuota'), rate: t('guide.teamRate'), desc: t('guide.teamDesc') },
            { name: t('guide.enterprise'), quota: t('guide.enterpriseQuota'), rate: t('guide.enterpriseRate'), desc: t('guide.enterpriseDesc') },
          ].map((plan) => (
            <div key={plan.name} className="guide-plan">
              <h4 className="guide-plan-name">{plan.name}</h4>
              <p className="guide-plan-desc">{plan.desc}</p>
              <div className="guide-plan-details">
                <span>{plan.quota}</span>
                <span>{plan.rate}</span>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
