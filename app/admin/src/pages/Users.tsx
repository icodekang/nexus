/**
 * @file Users - 用户管理页面
 * 展示用户列表，支持搜索和编辑用户信息（手机号、订阅套餐）
 */
import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import { fetchUsers, updateUser, type AdminUser } from '../api/admin';

// 订阅套餐颜色映射
const planColors: Record<string, string> = {
  yearly: '#6366F1',
  monthly: '#3B82F6',
  team: '#F59E0B',
  enterprise: '#EC4899',
  none: '#A1A1AA',
};

/**
 * Users - 用户管理主组件
 * @description 获取用户列表，支持搜索（带防抖），编辑用户手机号和套餐
 */
export default function Users() {
  const { t } = useI18n();
  const [searchQuery, setSearchQuery] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [editUser, setEditUser] = useState<AdminUser | null>(null);
  const [formPhone, setFormPhone] = useState('');
  const [formPlan, setFormPlan] = useState('monthly');

  // 防抖搜索：300ms 延迟更新搜索关键词，避免频繁请求
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(searchQuery), 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(searchQuery), 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const loadUsers = useCallback(() => {
    setLoading(true);
    fetchUsers(1, 50, debouncedSearch)
      .then((res) => {
        setUsers(res.data);
        setTotal(res.total);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [debouncedSearch]);

  useEffect(() => {
    loadUsers();
  }, [loadUsers]);

  // 打开编辑弹窗，填充用户现有数据
  const openEditModal = (user: AdminUser) => {
    setFormPhone(user.phone || '');
    setFormPlan(user.subscription_plan);
    setEditUser(user);
  };

  // 提交编辑用户表单
  const handleEditUser = async () => {
    if (!editUser) return;
    try {
      await updateUser(editUser.id, {
        phone: formPhone,
        subscription_plan: formPlan,
      });
      setEditUser(null);
      loadUsers();
    } catch {
      // Error handling
    }
  };

  return (
    <div style={styles.container}>
      {/* 页面头部：标题 + 用户数量 + 搜索框 */}
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('users.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? 'Loading...' : t('users.count', { count: total })}
          </p>
        </div>
        <div style={styles.headerActions}>
          {/* 搜索框：支持按邮箱搜索用户 */}
          <div style={styles.searchBox}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="11" cy="11" r="8" />
              <path d="M21 21l-4.35-4.35" />
            </svg>
            <input
              type="text"
              placeholder={t('users.searchPlaceholder')}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={styles.searchInput}
            />
          </div>
        </div>
      </header>

      {/* 用户列表表格 */}
      <div style={styles.tableCard}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={{ ...styles.th, paddingLeft: '20px' }}>{t('users.thUser')}</th>
              <th style={styles.th}>{t('users.thPhone')}</th>
              <th style={styles.th}>{t('users.thPlan')}</th>
              <th style={styles.th}>{t('users.thStatus')}</th>
              <th style={styles.th}>{t('users.thCreated')}</th>
              <th style={{ ...styles.th, paddingRight: '20px', textAlign: 'right' }}></th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.id} style={styles.tr}>
                {/* 用户邮箱列 */}
                <td style={{ ...styles.td, paddingLeft: '20px' }}>
                  <div style={styles.userCell}>
                    <div style={styles.userAvatar}>{user.email.charAt(0).toUpperCase()}</div>
                    <span style={styles.email}>{user.email}</span>
                  </div>
                </td>
                <td style={styles.td}>
                  <span style={styles.phone}>{user.phone || '-'}</span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.planBadge,
                    color: planColors[user.subscription_plan] || '#A1A1AA',
                    backgroundColor: `${planColors[user.subscription_plan] || '#A1A1AA'}12`,
                  }}>
                    {user.subscription_plan}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={{
                    ...styles.status,
                    color: user.is_active ? '#22C55E' : '#EF4444',
                  }}>
                    <span style={{
                      ...styles.statusDot,
                      backgroundColor: user.is_active ? '#22C55E' : '#EF4444',
                    }} />
                    {user.is_active ? t('common.active') : t('common.inactive')}
                  </span>
                </td>
                <td style={styles.td}>
                  <span style={styles.date}>{user.created_at.slice(0, 10)}</span>
                </td>
                <td style={{ ...styles.td, paddingRight: '20px', textAlign: 'right' }}>
                  <button style={styles.actionBtn} onClick={() => openEditModal(user)}>
                    {t('common.edit')}
                  </button>
                </td>
              </tr>
            ))}
            {!loading && users.length === 0 && (
              <tr>
                <td colSpan={6} style={{ ...styles.td, textAlign: 'center', color: '#A1A1AA', padding: '40px' }}>
                  {debouncedSearch ? 'No matching users found' : 'No users yet'}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <Modal open={!!editUser} onClose={() => setEditUser(null)} title={t('users.editUser')}>
        <div style={formStyles.form}>
          <div style={formStyles.field}>
            <label style={formStyles.label}>{t('users.phoneLabel')}</label>
            <input
              type="text"
              value={formPhone}
              onChange={(e) => setFormPhone(e.target.value)}
              placeholder={t('users.phonePlaceholder')}
              style={formStyles.input}
            />
          </div>
          <div style={formStyles.field}>
            <label style={formStyles.label}>{t('users.planLabel')}</label>
            <select value={formPlan} onChange={(e) => setFormPlan(e.target.value)} style={formStyles.input}>
              <option value="none">None</option>
              <option value="monthly">Monthly</option>
              <option value="yearly">Yearly</option>
              <option value="team">Team</option>
              <option value="enterprise">Enterprise</option>
            </select>
          </div>
          <div style={formStyles.actions}>
            <button style={formStyles.cancelBtn} onClick={() => setEditUser(null)}>{t('common.cancel')}</button>
            <button style={formStyles.submitBtn} onClick={handleEditUser}>{t('common.save')}</button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

const formStyles: Record<string, React.CSSProperties> = {
  form: { display: 'flex', flexDirection: 'column', gap: '16px' },
  field: { display: 'flex', flexDirection: 'column', gap: '6px' },
  label: { fontSize: '12px', fontWeight: '500', color: '#71717A', fontFamily: "'DM Sans', sans-serif" },
  input: {
    padding: '10px 12px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
    backgroundColor: '#FFFFFF',
    color: '#18181B',
    outline: 'none',
  },
  actions: { display: 'flex', justifyContent: 'flex-end', gap: '8px', marginTop: '8px' },
  cancelBtn: {
    padding: '8px 16px', borderRadius: '8px', border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF', fontSize: '12px', fontWeight: '500', color: '#71717A',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
  submitBtn: {
    padding: '8px 16px', borderRadius: '8px', border: 'none',
    backgroundColor: '#6366F1', fontSize: '12px', fontWeight: '500', color: '#FFFFFF',
    cursor: 'pointer', fontFamily: "'DM Sans', sans-serif",
  },
};

const styles: Record<string, React.CSSProperties> = {
  container: { maxWidth: '1200px' },
  header: {
    display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: '24px',
  },
  pageTitle: {
    fontSize: '24px', fontWeight: '700', color: '#18181B', margin: 0,
    fontFamily: "'Instrument Sans', sans-serif", letterSpacing: '-0.02em',
  },
  pageSubtitle: { fontSize: '13px', color: '#71717A', marginTop: '4px', fontFamily: "'DM Sans', sans-serif" },
  headerActions: { display: 'flex', gap: '10px' },
  searchBox: {
    display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 12px',
    backgroundColor: '#FFFFFF', borderRadius: '10px', border: '1px solid #E7E5E4',
    width: '200px', boxShadow: '0 1px 2px rgba(0,0,0,0.04)',
  },
  searchInput: {
    flex: 1, border: 'none', outline: 'none', fontSize: '12px',
    fontFamily: "'DM Sans', sans-serif", backgroundColor: 'transparent', color: '#18181B',
  },
  tableCard: {
    backgroundColor: '#FFFFFF', borderRadius: '14px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.04)', overflow: 'hidden',
  },
  table: { width: '100%', borderCollapse: 'collapse' },
  th: {
    padding: '12px 16px', textAlign: 'left', fontSize: '11px', fontWeight: '500',
    color: '#A1A1AA', textTransform: 'uppercase', letterSpacing: '0.04em',
    fontFamily: "'DM Sans', sans-serif", borderBottom: '1px solid #F5F5F4',
  },
  tr: { borderBottom: '1px solid #F5F5F4', transition: 'background 0.1s ease' },
  td: { padding: '14px 16px', fontSize: '13px', fontFamily: "'DM Sans', sans-serif" },
  userCell: { display: 'flex', alignItems: 'center', gap: '10px' },
  userAvatar: {
    width: '28px', height: '28px', borderRadius: '8px', backgroundColor: '#F5F5F4',
    display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '11px',
    fontWeight: '600', color: '#71717A', fontFamily: "'Instrument Sans', sans-serif", flexShrink: 0,
  },
  email: { fontWeight: '500', color: '#18181B' },
  phone: { color: '#71717A' },
  planBadge: {
    fontSize: '11px', fontWeight: '500', padding: '3px 10px',
    borderRadius: '9999px', fontFamily: "'DM Sans', sans-serif",
  },
  status: {
    display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px',
    fontWeight: '500', fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: { width: '5px', height: '5px', borderRadius: '50%', flexShrink: 0 },
  date: { color: '#A1A1AA', fontSize: '12px' },
  actionBtn: {
    padding: '5px 12px', backgroundColor: 'transparent', border: '1px solid #E7E5E4',
    borderRadius: '8px', fontSize: '11px', color: '#71717A', cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif", transition: 'all 0.1s ease',
  },
};
