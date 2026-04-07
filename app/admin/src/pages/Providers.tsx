export default function Providers() {
  const providers = [
    { id: '1', name: 'OpenAI', slug: 'openai', priority: 1, status: 'Active', models: 12 },
    { id: '2', name: 'Anthropic', slug: 'anthropic', priority: 2, status: 'Active', models: 8 },
    { id: '3', name: 'Google', slug: 'google', priority: 3, status: 'Active', models: 6 },
    { id: '4', name: 'DeepSeek', slug: 'deepseek', priority: 4, status: 'Active', models: 5 },
  ];

  return (
    <div>
      <div style={styles.header}>
        <h1 style={styles.pageTitle}>Providers</h1>
        <button style={styles.addButton}>+ Add Provider</button>
      </div>
      
      <div style={styles.grid}>
        {providers.map((provider) => (
          <div key={provider.id} style={styles.providerCard}>
            <div style={styles.providerHeader}>
              <div style={styles.providerInfo}>
                <div style={styles.providerIcon}>
                  {provider.name === 'OpenAI' && '🤖'}
                  {provider.name === 'Anthropic' && '🧠'}
                  {provider.name === 'Google' && '🔵'}
                  {provider.name === 'DeepSeek' && '🔴'}
                </div>
                <div>
                  <div style={styles.providerName}>{provider.name}</div>
                  <div style={styles.providerSlug}>{provider.slug}</div>
                </div>
              </div>
              <span style={[styles.statusBadge, provider.status === 'Active' ? styles.statusActive : styles.statusInactive]}>
                {provider.status}
              </span>
            </div>
            
            <div style={styles.providerStats}>
              <div style={styles.stat}>
                <div style={styles.statValue}>{provider.models}</div>
                <div style={styles.statLabel}>Models</div>
              </div>
              <div style={styles.stat}>
                <div style={styles.statValue}>#{provider.priority}</div>
                <div style={styles.statLabel}>Priority</div>
              </div>
            </div>
            
            <div style={styles.providerActions}>
              <button style={styles.actionButton}>Edit</button>
              <button style={styles.actionButtonDanger}>Disable</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
  },
  pageTitle: {
    fontSize: '28px',
    fontWeight: '700',
    color: '#1d1d1f',
  },
  addButton: {
    padding: '10px 20px',
    backgroundColor: '#10a37f',
    color: '#fff',
    border: 'none',
    borderRadius: '8px',
    fontSize: '14px',
    fontWeight: '600',
    cursor: 'pointer',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '20px',
  },
  providerCard: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    padding: '20px',
  },
  providerHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: '20px',
  },
  providerInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  providerIcon: {
    fontSize: '32px',
  },
  providerName: {
    fontSize: '18px',
    fontWeight: '600',
    color: '#1d1d1f',
  },
  providerSlug: {
    fontSize: '14px',
    color: '#86868b',
  },
  statusBadge: {
    padding: '4px 10px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
  },
  statusActive: {
    backgroundColor: '#dcfce7',
    color: '#16a34a',
  },
  statusInactive: {
    backgroundColor: '#fee2e2',
    color: '#dc2626',
  },
  providerStats: {
    display: 'flex',
    gap: '32px',
    marginBottom: '20px',
  },
  stat: {
    display: 'flex',
    flexDirection: 'column',
  },
  statValue: {
    fontSize: '20px',
    fontWeight: '700',
    color: '#1d1d1f',
  },
  statLabel: {
    fontSize: '12px',
    color: '#86868b',
    marginTop: '2px',
  },
  providerActions: {
    display: 'flex',
    gap: '8px',
  },
  actionButton: {
    flex: 1,
    padding: '10px',
    backgroundColor: '#f5f5f7',
    border: 'none',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: '500',
    cursor: 'pointer',
  },
  actionButtonDanger: {
    flex: 1,
    padding: '10px',
    backgroundColor: '#fee2e2',
    border: 'none',
    borderRadius: '6px',
    fontSize: '13px',
    fontWeight: '500',
    color: '#dc2626',
    cursor: 'pointer',
  },
};
