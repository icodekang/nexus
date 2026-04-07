export default function Models() {
  const models = [
    { id: '1', name: 'GPT-4o', provider: 'OpenAI', contextWindow: 128000, capabilities: ['vision', 'function_call'], status: 'Active' },
    { id: '2', name: 'GPT-4o Mini', provider: 'OpenAI', contextWindow: 128000, capabilities: ['function_call'], status: 'Active' },
    { id: '3', name: 'Claude 3.5 Sonnet', provider: 'Anthropic', contextWindow: 200000, capabilities: ['vision'], status: 'Active' },
    { id: '4', name: 'Gemini 1.5 Pro', provider: 'Google', contextWindow: 2000000, capabilities: ['vision'], status: 'Active' },
    { id: '5', name: 'DeepSeek V3', provider: 'DeepSeek', contextWindow: 64000, capabilities: [], status: 'Active' },
  ];

  return (
    <div>
      <div style={styles.header}>
        <h1 style={styles.pageTitle}>Models</h1>
        <button style={styles.addButton}>+ Add Model</button>
      </div>
      
      <div style={styles.table}>
        <div style={styles.tableHeader}>
          <div style={styles.tableCell}>Model</div>
          <div style={styles.tableCell}>Provider</div>
          <div style={styles.tableCell}>Context Window</div>
          <div style={styles.tableCell}>Capabilities</div>
          <div style={styles.tableCell}>Status</div>
          <div style={styles.tableCell}>Actions</div>
        </div>
        {models.map((model) => (
          <div key={model.id} style={styles.tableRow}>
            <div style={styles.tableCell}>
              <div style={styles.modelName}>{model.name}</div>
            </div>
            <div style={styles.tableCell}>
              <span style={styles.providerBadge}>{model.provider}</span>
            </div>
            <div style={styles.tableCell}>{(model.contextWindow / 1000).toFixed(0)}K</div>
            <div style={styles.tableCell}>
              <div style={styles.capabilities}>
                {model.capabilities.map((cap) => (
                  <span key={cap} style={styles.capabilityTag}>{cap}</span>
                ))}
                {model.capabilities.length === 0 && <span style={styles.noCapabilities}>-</span>}
              </div>
            </div>
            <div style={styles.tableCell}>
              <span style={[styles.statusBadge, model.status === 'Active' ? styles.statusActive : styles.statusInactive]}>
                {model.status}
              </span>
            </div>
            <div style={styles.tableCell}>
              <button style={styles.actionButton}>Edit</button>
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
  table: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    overflow: 'hidden',
  },
  tableHeader: {
    display: 'grid',
    gridTemplateColumns: '1.5fr 1fr 1fr 1.5fr 1fr 1fr',
    padding: '16px 20px',
    backgroundColor: '#f5f5f7',
    fontSize: '12px',
    fontWeight: '600',
    color: '#86868b',
    textTransform: 'uppercase',
  },
  tableRow: {
    display: 'grid',
    gridTemplateColumns: '1.5fr 1fr 1fr 1.5fr 1fr 1fr',
    padding: '16px 20px',
    borderBottom: '1px solid #f5f5f7',
    alignItems: 'center',
  },
  tableCell: {
    fontSize: '14px',
    color: '#1d1d1f',
  },
  modelName: {
    fontWeight: '600',
  },
  providerBadge: {
    backgroundColor: '#f0f0f0',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
  },
  capabilities: {
    display: 'flex',
    gap: '4px',
    flexWrap: 'wrap',
  },
  capabilityTag: {
    backgroundColor: '#e8f5ef',
    color: '#10a37f',
    padding: '2px 6px',
    borderRadius: '4px',
    fontSize: '11px',
    fontWeight: '500',
  },
  noCapabilities: {
    color: '#86868b',
  },
  statusBadge: {
    padding: '4px 8px',
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
  actionButton: {
    padding: '6px 12px',
    backgroundColor: '#f5f5f7',
    border: 'none',
    borderRadius: '6px',
    fontSize: '12px',
    fontWeight: '500',
    cursor: 'pointer',
  },
};
