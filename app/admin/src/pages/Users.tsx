export default function Users() {
  const users = [
    { id: '1', email: 'user@example.com', phone: '+86 138****8888', plan: 'Yearly', status: 'Active', created: '2024-01-01' },
    { id: '2', email: 'john@example.com', phone: '+86 139****6666', plan: 'Monthly', status: 'Active', created: '2024-01-05' },
    { id: '3', email: 'jane@example.com', phone: '+86 137****5555', plan: 'Team', status: 'Active', created: '2024-01-10' },
    { id: '4', email: 'bob@example.com', phone: '+86 136****4444', plan: 'None', status: 'Expired', created: '2023-12-01' },
  ];

  return (
    <div>
      <div style={styles.header}>
        <h1 style={styles.pageTitle}>Users</h1>
        <input 
          type="text" 
          placeholder="Search users..." 
          style={styles.searchInput}
        />
      </div>
      
      <div style={styles.table}>
        <div style={styles.tableHeader}>
          <div style={styles.tableCell}>User</div>
          <div style={styles.tableCell}>Phone</div>
          <div style={styles.tableCell}>Plan</div>
          <div style={styles.tableCell}>Status</div>
          <div style={styles.tableCell}>Created</div>
          <div style={styles.tableCell}>Actions</div>
        </div>
        {users.map((user) => (
          <div key={user.id} style={styles.tableRow}>
            <div style={styles.tableCell}>
              <div style={styles.userEmail}>{user.email}</div>
            </div>
            <div style={styles.tableCell}>{user.phone}</div>
            <div style={styles.tableCell}>
              <span style={styles.planBadge}>{user.plan}</span>
            </div>
            <div style={styles.tableCell}>
              <span style={[styles.statusBadge, user.status === 'Active' ? styles.statusActive : styles.statusExpired]}>
                {user.status}
              </span>
            </div>
            <div style={styles.tableCell}>{user.created}</div>
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
  searchInput: {
    padding: '10px 16px',
    borderRadius: '8px',
    border: '1px solid #e5e5e7',
    fontSize: '14px',
    width: '300px',
  },
  table: {
    backgroundColor: '#fff',
    borderRadius: '12px',
    overflow: 'hidden',
  },
  tableHeader: {
    display: 'grid',
    gridTemplateColumns: '2fr 1.5fr 1fr 1fr 1fr 1fr',
    padding: '16px 20px',
    backgroundColor: '#f5f5f7',
    fontSize: '12px',
    fontWeight: '600',
    color: '#86868b',
    textTransform: 'uppercase',
  },
  tableRow: {
    display: 'grid',
    gridTemplateColumns: '2fr 1.5fr 1fr 1fr 1fr 1fr',
    padding: '16px 20px',
    borderBottom: '1px solid #f5f5f7',
    alignItems: 'center',
  },
  tableCell: {
    fontSize: '14px',
    color: '#1d1d1f',
  },
  userEmail: {
    fontWeight: '500',
  },
  planBadge: {
    backgroundColor: '#e8f5ef',
    color: '#10a37f',
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '500',
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
  statusExpired: {
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
