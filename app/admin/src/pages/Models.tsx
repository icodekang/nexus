import { useI18n } from '../i18n';

export default function Models() {
  const { t } = useI18n();
  const models = [
    { id: '1', name: 'GPT-4o', provider: 'OpenAI', context: '128K', caps: ['vision', 'function'], providerColor: '#10A37F' },
    { id: '2', name: 'GPT-4o Mini', provider: 'OpenAI', context: '128K', caps: ['function'], providerColor: '#10A37F' },
    { id: '3', name: 'Claude 3.5 Sonnet', provider: 'Anthropic', context: '200K', caps: ['vision'], providerColor: '#D97706' },
    { id: '4', name: 'Gemini 1.5 Pro', provider: 'Google', context: '2M', caps: ['vision'], providerColor: '#4285F4' },
    { id: '5', name: 'DeepSeek V3', provider: 'DeepSeek', context: '64K', caps: [], providerColor: '#6366F1' },
  ];

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('models.title')}</h1>
          <p style={styles.pageSubtitle}>{t('models.subtitle', { count: models.length })}</p>
        </div>
        <button style={styles.addBtn}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          {t('models.addModel')}
        </button>
      </header>

      <div style={styles.tableCard}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={{ ...styles.th, paddingLeft: '20px' }}>{t('models.thModel')}</th>
              <th style={styles.th}>{t('models.thProvider')}</th>
              <th style={styles.th}>{t('models.thContext')}</th>
              <th style={styles.th}>{t('models.thCapabilities')}</th>
              <th style={{ ...styles.th, paddingRight: '20px', textAlign: 'right' }}></th>
            </tr>
          </thead>
          <tbody>
            {models.map((m) => (
              <tr key={m.id} style={styles.tr}>
                <td style={{ ...styles.td, paddingLeft: '20px' }}>
                  <span style={styles.modelName}>{m.name}</span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.providerBadge,
                    color: m.providerColor,
                    backgroundColor: `${m.providerColor}12`,
                  }}>
                    {m.provider}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={styles.context}>{m.context}</span>
                </td>
                <td style={styles.td}>
                  <div style={styles.caps}>
                    {m.caps.length > 0 ? m.caps.map((c) => (
                      <span key={c} style={styles.capTag}>{c}</span>
                    )) : <span style={styles.noCap}>-</span>}
                  </div>
                </td>
                <td style={{ ...styles.td, paddingRight: '20px', textAlign: 'right' }}>
                  <button style={styles.actionBtn}>{t('common.edit')}</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
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
  modelName: { fontWeight: '500', color: '#18181B' },
  providerBadge: {
    fontSize: '11px',
    fontWeight: '500',
    padding: '3px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  context: { color: '#71717A', fontSize: '12px' },
  caps: { display: 'flex', gap: '4px' },
  capTag: {
    fontSize: '10px',
    fontWeight: '500',
    color: '#71717A',
    backgroundColor: '#F5F5F4',
    padding: '2px 8px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  noCap: { color: '#A1A1AA' },
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
};
