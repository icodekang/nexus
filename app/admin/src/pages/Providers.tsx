import { useI18n } from '../i18n';

export default function Providers() {
  const { t } = useI18n();
  const providers = [
    { id: '1', name: 'OpenAI', slug: 'openai', models: 12, priority: 1, status: 'Active', color: '#10A37F' },
    { id: '2', name: 'Anthropic', slug: 'anthropic', models: 8, priority: 2, status: 'Active', color: '#D97706' },
    { id: '3', name: 'Google', slug: 'google', models: 6, priority: 3, status: 'Active', color: '#4285F4' },
    { id: '4', name: 'DeepSeek', slug: 'deepseek', models: 5, priority: 4, status: 'Active', color: '#6366F1' },
  ];

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('providers.title')}</h1>
          <p style={styles.pageSubtitle}>{t('providers.subtitle')}</p>
        </div>
        <button style={styles.addBtn}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          {t('providers.addProvider')}
        </button>
      </header>

      <div style={styles.grid}>
        {providers.map((p) => (
          <div key={p.id} style={styles.card}>
            <div style={styles.cardTop}>
              <div style={{ ...styles.logo, backgroundColor: `${p.color}14`, color: p.color }}>
                {p.name.charAt(0)}
              </div>
              <div style={styles.info}>
                <h3 style={styles.name}>{p.name}</h3>
                <span style={styles.slug}>{p.slug}</span>
              </div>
              <span style={styles.statusBadge}>
                <span style={{ ...styles.statusDot, backgroundColor: '#22C55E' }} />
                {t('common.active')}
              </span>
            </div>
            <div style={styles.stats}>
              <div style={styles.stat}>
                <span style={styles.statValue}>{p.models}</span>
                <span style={styles.statLabel}>{t('providers.models')}</span>
              </div>
              <div style={styles.statDivider} />
              <div style={styles.stat}>
                <span style={styles.statValue}>#{p.priority}</span>
                <span style={styles.statLabel}>{t('providers.priority')}</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

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
  addBtn: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 14px',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    border: 'none',
    borderRadius: '10px',
    fontSize: '12px',
    fontWeight: '500',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '14px',
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: '14px',
    padding: '20px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  cardTop: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  logo: {
    width: '40px',
    height: '40px',
    borderRadius: '10px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '16px',
    fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
    flexShrink: 0,
  },
  info: {
    flex: 1,
  },
  name: {
    fontSize: '14px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  slug: {
    fontSize: '12px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusBadge: {
    display: 'flex',
    alignItems: 'center',
    gap: '5px',
    fontSize: '11px',
    fontWeight: '500',
    color: '#22C55E',
    backgroundColor: 'rgba(34, 197, 94, 0.08)',
    padding: '4px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: {
    width: '5px',
    height: '5px',
    borderRadius: '50%',
  },
  stats: {
    display: 'flex',
    alignItems: 'center',
    paddingTop: '14px',
    borderTop: '1px solid #F5F5F4',
  },
  stat: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '2px',
  },
  statValue: {
    fontSize: '16px',
    fontWeight: '700',
    color: '#18181B',
    fontFamily: "'Instrument Sans', sans-serif",
  },
  statLabel: {
    fontSize: '10px',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    fontFamily: "'DM Sans', sans-serif",
  },
  statDivider: {
    width: '1px',
    height: '24px',
    backgroundColor: '#F5F5F4',
  },
};
