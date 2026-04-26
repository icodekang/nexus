/**
 * @file ProviderKeys - 提供商密钥管理页面
 * 管理 AI 服务提供商的 API 密钥，支持添加、编辑、删除和测试密钥
 * 支持查看所有用户 API 密钥，支持按用户邮箱过滤
 */
import { useState, useEffect, useCallback } from 'react';
import { useI18n } from '../i18n';
import Modal from '../components/Modal';
import {
  fetchProviderKeys, createProviderKey, updateProviderKey, deleteProviderKey, testProviderKey,
  fetchProviders, fetchUserApiKeys, deleteUserApiKey,
  type ProviderKey, type AdminProvider, type UserApiKey,
} from '../api/admin';
import { Key, Trash2, Eye, EyeOff, Users, Server } from 'lucide-react';

// 提供商品牌颜色映射
const providerColors: Record<string, string> = {
  openai: '#10A37F',
  anthropic: '#D97706',
  google: '#4285F4',
  deepseek: '#6366F1',
};

// 根据 slug 获取提供商颜色
function getColor(slug: string): string {
  return providerColors[slug] || '#A1A1AA';
}

/**
 * ProviderKeys - 密钥管理主组件
 * @description 获取密钥列表，支持添加/编辑/删除/测试密钥
 */
export default function ProviderKeys() {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<'provider' | 'user'>('provider');
  const [keys, setKeys] = useState<ProviderKey[]>([]);
  const [userKeys, setUserKeys] = useState<UserApiKey[]>([]);
  const [providers, setProviders] = useState<AdminProvider[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [editKey, setEditKey] = useState<ProviderKey | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ProviderKey | null>(null);
  const [deleteUserKeyTarget, setDeleteUserKeyTarget] = useState<UserApiKey | null>(null);
  // 密钥显示/隐藏状态集合
  const [visibleKeys, setVisibleKeys] = useState<Set<string>>(new Set());
  // 当前正在测试的密钥 ID
  const [testingId, setTestingId] = useState<string | null>(null);
  // 测试结果
  const [testResult, setTestResult] = useState<{ id: string; success: boolean; message?: string } | null>(null);
  // 用户过滤
  const [userFilter, setUserFilter] = useState('');

  // 表单状态
  const [formProvider, setFormProvider] = useState('');
  const [formApiKey, setFormApiKey] = useState('');
  const [formBaseUrl, setFormBaseUrl] = useState('');
  const [formPriority, setFormPriority] = useState('1');

  // 加载密钥和提供商数据
  const loadData = useCallback(() => {
    setLoading(true);
    Promise.all([fetchProviderKeys(), fetchProviders()])
      .then(([keysRes, providersRes]) => {
        setKeys(keysRes.data);
        setProviders(providersRes.data);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  // 加载用户 API 密钥
  const loadUserKeys = useCallback(() => {
    setLoading(true);
    fetchUserApiKeys(undefined, userFilter || undefined)
      .then((res) => setUserKeys(res.data))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [userFilter]);

  useEffect(() => {
    if (activeTab === 'provider') {
      loadData();
    } else {
      loadUserKeys();
    }
  }, [activeTab, loadData, loadUserKeys]);

  // 用户过滤搜索
  useEffect(() => {
    if (activeTab === 'user') {
      const timer = setTimeout(() => {
        loadUserKeys();
      }, 300);
      return () => clearTimeout(timer);
    }
  }, [userFilter, activeTab, loadUserKeys]);

  const resetForm = () => {
    setFormProvider('');
    setFormApiKey('');
    setFormBaseUrl('');
    setFormPriority('1');
  };

  // 打开添加密钥弹窗
  const openAddModal = () => {
    resetForm();
    setShowAddModal(true);
  };

  // 打开编辑弹窗（不填 API key 字段，仅编辑 base_url 和 priority）
  const openEditModal = (k: ProviderKey) => {
    setFormProvider(k.provider_slug);
    setFormApiKey('');
    setFormBaseUrl(k.base_url);
    setFormPriority(String(k.priority));
    setEditKey(k);
  };

  // 添加新密钥
  const handleAdd = async () => {
    if (!formProvider || !formApiKey.trim()) return;
    try {
      await createProviderKey({
        provider_slug: formProvider,
        api_key: formApiKey,
        base_url: formBaseUrl || undefined,
        priority: parseInt(formPriority) || undefined,
      });
      setShowAddModal(false);
      loadData();
    } catch {
      // Error handling
    }
  };

  // 编辑密钥（仅更新 base_url 和 priority，如填写了 api_key 则同时更新）
  const handleEdit = async () => {
    if (!editKey) return;
    try {
      const data: Record<string, unknown> = {
        base_url: formBaseUrl,
        priority: parseInt(formPriority) || editKey.priority,
      };
      if (formApiKey.trim()) {
        data.api_key = formApiKey;
      }
      await updateProviderKey(editKey.id, data as Parameters<typeof updateProviderKey>[1]);
      setEditKey(null);
      loadData();
    } catch {
      // Error handling
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteProviderKey(deleteTarget.id);
      setDeleteTarget(null);
      loadData();
    } catch {
      // Error handling
    }
  };

  const handleDeleteUserKey = async () => {
    if (!deleteUserKeyTarget) return;
    try {
      await deleteUserApiKey(deleteUserKeyTarget.id);
      setDeleteUserKeyTarget(null);
      loadUserKeys();
    } catch {
      // Error handling
    }
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    setTestResult(null);
    try {
      const res = await testProviderKey(id);
      setTestResult({ id, success: res.success, message: res.message });
    } catch {
      setTestResult({ id, success: false, message: 'Connection failed' });
    } finally {
      setTestingId(null);
      setTimeout(() => setTestResult(null), 5000);
    }
  };

  const toggleVisibility = (id: string) => {
    setVisibleKeys((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div>
          <h1 style={styles.pageTitle}>{t('providerKeys.title')}</h1>
          <p style={styles.pageSubtitle}>
            {loading ? 'Loading...' : t('providerKeys.subtitle')}
          </p>
        </div>
      </header>

      {/* Tabs */}
      <div style={styles.tabs}>
        <button
          style={{ ...styles.tab, ...(activeTab === 'provider' ? styles.tabActive : {}) }}
          onClick={() => setActiveTab('provider')}
        >
          <Server size={14} />
          {t('providerKeys.tabProvider')}
        </button>
        <button
          style={{ ...styles.tab, ...(activeTab === 'user' ? styles.tabActive : {}) }}
          onClick={() => setActiveTab('user')}
        >
          <Users size={14} />
          {t('providerKeys.tabUser')}
        </button>
      </div>

      {/* User Keys Filter */}
      {activeTab === 'user' && (
        <div style={styles.filterBar}>
          <input
            type="text"
            placeholder={t('providerKeys.filterByEmail')}
            value={userFilter}
            onChange={(e) => setUserFilter(e.target.value)}
            style={styles.filterInput}
          />
        </div>
      )}

      {/* Table */}
      <div style={styles.tableCard}>
        <table style={styles.table}>
          <thead>
            <tr>
              <th style={styles.th}>{t('providerKeys.thProvider')}</th>
              <th style={styles.th}>{t('providerKeys.thKey')}</th>
              <th style={styles.th}>{t('providerKeys.thUrl')}</th>
              <th style={styles.th}>{t('providerKeys.thStatus')}</th>
              <th style={styles.th}>{t('providerKeys.thPriority')}</th>
              <th style={{ ...styles.th, textAlign: 'right' }}>{t('providerKeys.thActions')}</th>
            </tr>
          </thead>
          <tbody>
            {keys.map((k) => {
              const color = getColor(k.provider_slug);
              const isVisible = visibleKeys.has(k.id);
              const isTesting = testingId === k.id;
              const testRes = testResult?.id === k.id ? testResult : null;

              return (
                <tr key={k.id} style={styles.tr}>
                  {/* Provider */}
                  <td style={styles.td}>
                    <div style={styles.providerCell}>
                      <div style={{ ...styles.providerDot, backgroundColor: `${color}18`, color }}>
                        {k.provider_slug.charAt(0).toUpperCase()}
                      </div>
                      <span style={styles.providerName}>{k.provider_slug}</span>
                    </div>
                  </td>

                  {/* API Key */}
                  <td style={styles.td}>
                    <div style={styles.keyCell}>
                      <code style={styles.keyCode}>
                        {isVisible ? k.api_key_preview : k.api_key_masked}
                      </code>
                      <button
                        style={styles.eyeBtn}
                        onClick={() => toggleVisibility(k.id)}
                        title={isVisible ? t('providerKeys.hideKey') : t('providerKeys.revealKey')}
                      >
                        {isVisible ? (
                          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
                            <line x1="1" y1="1" x2="23" y2="23" />
                          </svg>
                        ) : (
                          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                            <circle cx="12" cy="12" r="3" />
                          </svg>
                        )}
                      </button>
                    </div>
                  </td>

                  {/* Base URL */}
                  <td style={styles.td}>
                    <span style={styles.urlText} title={k.base_url}>
                      {k.base_url.length > 28 ? k.base_url.slice(0, 28) + '...' : k.base_url}
                    </span>
                  </td>

                  {/* Status */}
                  <td style={styles.td}>
                    <span style={{
                      ...styles.statusBadge,
                      color: k.is_active ? '#22C55E' : '#EF4444',
                      backgroundColor: k.is_active ? 'rgba(34, 197, 94, 0.08)' : 'rgba(239, 68, 68, 0.08)',
                    }}>
                      <span style={{
                        ...styles.statusDot,
                        backgroundColor: k.is_active ? '#22C55E' : '#EF4444',
                      }} />
                      {k.is_active ? t('common.active') : t('common.inactive')}
                    </span>
                  </td>

                  {/* Priority */}
                  <td style={styles.td}>
                    <span style={styles.priorityBadge}>#{k.priority}</span>
                  </td>

                  {/* Actions */}
                  <td style={{ ...styles.td, textAlign: 'right' }}>
                    <div style={styles.actions}>
                      {/* Test button */}
                      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '4px' }}>
                        <button
                          style={{
                            ...styles.actionBtn,
                            ...(testRes?.success ? styles.testSuccessBtn : {}),
                            ...(testRes && !testRes.success ? styles.testFailBtn : {}),
                          }}
                          onClick={() => handleTest(k.id)}
                          disabled={isTesting}
                        >
                          {isTesting ? t('providerKeys.testing') : testRes
                            ? (testRes.success ? t('providerKeys.testSuccess') : t('providerKeys.testFailed'))
                            : t('providerKeys.testKey')
                          }
                        </button>
                        {testRes && !testRes.success && testRes.message && (
                          <span style={{ fontSize: '11px', color: '#EF4444', maxWidth: '150px', textAlign: 'right' }}>
                            {testRes.message}
                          </span>
                        )}
                      </div>
                      <button style={styles.actionBtn} onClick={() => openEditModal(k)}>
                        {t('common.edit')}
                      </button>
                      <button
                        style={{ ...styles.actionBtn, color: '#EF4444' }}
                        onClick={() => setDeleteTarget(k)}
                      >
                        {t('common.delete')}
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>

        {!loading && keys.length === 0 && (
          <div style={styles.empty}>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" style={{ marginBottom: '12px' }}>
              <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
            </svg>
            <h3 style={styles.emptyTitle}>{t('providerKeys.noKeys')}</h3>
            <p style={styles.emptyDesc}>{t('providerKeys.noKeysDesc')}</p>
          </div>
        )}
      </div>

      {/* User API Keys Table */}
      {activeTab === 'user' && (
        <div style={styles.tableCard}>
          <table style={styles.table}>
            <thead>
              <tr>
                <th style={styles.th}>{t('providerKeys.thUser')}</th>
                <th style={styles.th}>{t('providerKeys.thKeyName')}</th>
                <th style={styles.th}>{t('providerKeys.thKey')}</th>
                <th style={styles.th}>{t('providerKeys.thStatus')}</th>
                <th style={styles.th}>{t('providerKeys.thLastUsed')}</th>
                <th style={styles.th}>{t('providerKeys.thCreated')}</th>
                <th style={{ ...styles.th, textAlign: 'right' }}>{t('providerKeys.thActions')}</th>
              </tr>
            </thead>
            <tbody>
              {userKeys.map((k) => (
                <tr key={k.id} style={styles.tr}>
                  <td style={styles.td}>
                    <div style={styles.userCell}>
                      <div style={styles.userAvatar}>{k.user_email.charAt(0).toUpperCase()}</div>
                      <span style={styles.userEmail}>{k.user_email}</span>
                    </div>
                  </td>
                  <td style={styles.td}>
                    <span style={styles.keyName}>{k.name || t('providerKeys.unnamed')}</span>
                  </td>
                  <td style={styles.td}>
                    <code style={styles.keyCode}>{k.key_prefix}••••••</code>
                  </td>
                  <td style={styles.td}>
                    <span style={{
                      ...styles.statusBadge,
                      color: k.is_active ? '#22C55E' : '#EF4444',
                      backgroundColor: k.is_active ? 'rgba(34, 197, 94, 0.08)' : 'rgba(239, 68, 68, 0.08)',
                    }}>
                      <span style={{ ...styles.statusDot, backgroundColor: k.is_active ? '#22C55E' : '#EF4444' }} />
                      {k.is_active ? t('common.active') : t('common.inactive')}
                    </span>
                  </td>
                  <td style={styles.td}>
                    <span style={styles.dateText}>
                      {k.last_used_at ? new Date(k.last_used_at).toLocaleDateString() : '-'}
                    </span>
                  </td>
                  <td style={styles.td}>
                    <span style={styles.dateText}>
                      {new Date(k.created_at).toLocaleDateString()}
                    </span>
                  </td>
                  <td style={{ ...styles.td, textAlign: 'right' }}>
                    <button
                      style={{ ...styles.actionBtn, color: '#EF4444' }}
                      onClick={() => setDeleteUserKeyTarget(k)}
                    >
                      <Trash2 size={14} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {!loading && userKeys.length === 0 && (
            <div style={styles.empty}>
              <Users size={28} strokeWidth={1.5} style={{ marginBottom: '12px', color: '#A8A29E' }} />
              <h3 style={styles.emptyTitle}>{t('providerKeys.noUserKeys')}</h3>
              <p style={styles.emptyDesc}>{t('providerKeys.noUserKeysDesc')}</p>
            </div>
          )}
        </div>
      )}

      {/* Add Modal */}
      <Modal open={showAddModal} onClose={() => setShowAddModal(false)} title={t('providerKeys.addKey')}>
        <KeyForm
          providers={providers}
          provider={formProvider} setProvider={setFormProvider}
          apiKey={formApiKey} setApiKey={setFormApiKey}
          baseUrl={formBaseUrl} setBaseUrl={setFormBaseUrl}
          priority={formPriority} setPriority={setFormPriority}
          t={t}
          onSubmit={handleAdd}
          onCancel={() => setShowAddModal(false)}
          submitLabel={t('providerKeys.addKey')}
          isEdit={false}
        />
      </Modal>

      {/* Edit Modal */}
      <Modal open={!!editKey} onClose={() => setEditKey(null)} title={t('providerKeys.editKey')}>
        <KeyForm
          providers={providers}
          provider={formProvider} setProvider={setFormProvider}
          apiKey={formApiKey} setApiKey={setFormApiKey}
          baseUrl={formBaseUrl} setBaseUrl={setFormBaseUrl}
          priority={formPriority} setPriority={setFormPriority}
          t={t}
          onSubmit={handleEdit}
          onCancel={() => setEditKey(null)}
          submitLabel={t('common.save')}
          isEdit
        />
      </Modal>

      {/* Delete Confirmation */}
      <Modal open={!!deleteTarget} onClose={() => setDeleteTarget(null)} title={t('common.delete')} width={380}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <p style={{ fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif" }}>
            {t('common.deleteConfirm')}
          </p>
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
            <button style={formStyles.cancelBtn} onClick={() => setDeleteTarget(null)}>
              {t('common.cancel')}
            </button>
            <button style={{ ...formStyles.submitBtn, backgroundColor: '#EF4444' }} onClick={handleDelete}>
              {t('common.delete')}
            </button>
          </div>
        </div>
      </Modal>

      {/* Delete User Key Confirmation */}
      <Modal open={!!deleteUserKeyTarget} onClose={() => setDeleteUserKeyTarget(null)} title={t('common.delete')} width={380}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <p style={{ fontSize: '13px', color: '#71717A', margin: 0, fontFamily: "'DM Sans', sans-serif" }}>
            {t('providerKeys.deleteUserKeyConfirm')}
          </p>
          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
            <button style={formStyles.cancelBtn} onClick={() => setDeleteUserKeyTarget(null)}>
              {t('common.cancel')}
            </button>
            <button style={{ ...formStyles.submitBtn, backgroundColor: '#EF4444' }} onClick={handleDeleteUserKey}>
              {t('common.delete')}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function KeyForm({
  providers, provider, setProvider, apiKey, setApiKey, baseUrl, setBaseUrl,
  priority, setPriority, t, onSubmit, onCancel, submitLabel, isEdit,
}: {
  providers: AdminProvider[];
  provider: string; setProvider: (v: string) => void;
  apiKey: string; setApiKey: (v: string) => void;
  baseUrl: string; setBaseUrl: (v: string) => void;
  priority: string; setPriority: (v: string) => void;
  t: (key: string) => string;
  onSubmit: () => void;
  onCancel: () => void;
  submitLabel: string;
  isEdit: boolean;
}) {
  return (
    <div style={formStyles.form}>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('providerKeys.providerLabel')}</label>
        {isEdit ? (
          <input type="text" value={provider} disabled style={{ ...formStyles.input, backgroundColor: '#F5F5F4', color: '#A1A1AA' }} />
        ) : (
          <select
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
            style={formStyles.input}
          >
            <option value="">{t('providerKeys.selectProvider')}</option>
            {providers.map((p) => (
              <option key={p.id} value={p.slug}>{p.name} ({p.slug})</option>
            ))}
            {/* Fallback options for known providers not yet in DB */}
            {['openai', 'anthropic', 'google', 'deepseek'].filter(
              (s) => !providers.find((p) => p.slug === s)
            ).map((s) => (
              <option key={s} value={s}>{s.charAt(0).toUpperCase() + s.slice(1)}</option>
            ))}
          </select>
        )}
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>
          {t('providerKeys.apiKeyLabel')}
          {isEdit && <span style={{ fontWeight: '400', marginLeft: '6px', color: '#A1A1AA' }}>({t('providerKeys.maskedKey')})</span>}
        </label>
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder={isEdit ? t('providerKeys.apiKeyPlaceholder') + ' (leave blank to keep)' : t('providerKeys.apiKeyPlaceholder')}
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('providerKeys.baseUrlLabel')}</label>
        <input
          type="text"
          value={baseUrl}
          onChange={(e) => setBaseUrl(e.target.value)}
          placeholder={t('providerKeys.baseUrlPlaceholder')}
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.field}>
        <label style={formStyles.label}>{t('providerKeys.priorityLabel')}</label>
        <input
          type="number"
          value={priority}
          onChange={(e) => setPriority(e.target.value)}
          min="1"
          style={formStyles.input}
        />
      </div>
      <div style={formStyles.actions}>
        <button style={formStyles.cancelBtn} onClick={onCancel}>{t('common.cancel')}</button>
        <button
          style={formStyles.submitBtn}
          onClick={onSubmit}
          disabled={!provider || (!isEdit && !apiKey.trim())}
        >
          {submitLabel}
        </button>
      </div>
    </div>
  );
}

const formStyles: Record<string, React.CSSProperties> = {
  form: { display: 'flex', flexDirection: 'column', gap: '16px' },
  field: { display: 'flex', flexDirection: 'column', gap: '6px' },
  label: {
    fontSize: '12px',
    fontWeight: '500',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
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
  actions: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '8px',
    marginTop: '8px',
  },
  cancelBtn: {
    padding: '8px 16px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    backgroundColor: '#FFFFFF',
    fontSize: '12px',
    fontWeight: '500',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
  submitBtn: {
    padding: '8px 16px',
    borderRadius: '8px',
    border: 'none',
    backgroundColor: '#6366F1',
    fontSize: '12px',
    fontWeight: '500',
    color: '#FFFFFF',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
};

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
  table: {
    width: '100%',
    borderCollapse: 'collapse',
    fontFamily: "'DM Sans', sans-serif",
  },
  th: {
    textAlign: 'left',
    padding: '12px 16px',
    fontSize: '11px',
    fontWeight: '500',
    color: '#A1A1AA',
    textTransform: 'uppercase',
    letterSpacing: '0.04em',
    borderBottom: '1px solid #F5F5F4',
    whiteSpace: 'nowrap',
  },
  tr: {
    borderBottom: '1px solid #F5F5F4',
    transition: 'background-color 0.1s ease',
  },
  td: {
    padding: '14px 16px',
    fontSize: '13px',
    color: '#18181B',
    verticalAlign: 'middle',
  },
  providerCell: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  providerDot: {
    width: '32px',
    height: '32px',
    borderRadius: '8px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '13px',
    fontWeight: '700',
    fontFamily: "'Instrument Sans', sans-serif",
    flexShrink: 0,
  },
  providerName: {
    fontSize: '13px',
    fontWeight: '500',
    color: '#18181B',
    textTransform: 'capitalize',
  },
  keyCell: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
  },
  keyCode: {
    fontSize: '12px',
    color: '#71717A',
    fontFamily: "'DM Sans', monospace",
    backgroundColor: '#F5F5F4',
    padding: '3px 8px',
    borderRadius: '5px',
  },
  eyeBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '24px',
    height: '24px',
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '5px',
    cursor: 'pointer',
    color: '#A1A1AA',
    transition: 'all 0.1s ease',
    flexShrink: 0,
  },
  urlText: {
    fontSize: '12px',
    color: '#A1A1AA',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusBadge: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '5px',
    fontSize: '11px',
    fontWeight: '500',
    padding: '4px 10px',
    borderRadius: '9999px',
    fontFamily: "'DM Sans', sans-serif",
  },
  statusDot: {
    width: '5px',
    height: '5px',
    borderRadius: '50%',
  },
  priorityBadge: {
    fontSize: '12px',
    fontWeight: '600',
    color: '#71717A',
    fontFamily: "'DM Sans', sans-serif",
  },
  actions: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    justifyContent: 'flex-end',
  },
  actionBtn: {
    padding: '5px 10px',
    backgroundColor: 'transparent',
    border: '1px solid #E7E5E4',
    borderRadius: '7px',
    fontSize: '11px',
    fontWeight: '500',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.1s ease',
    whiteSpace: 'nowrap',
  },
  testSuccessBtn: {
    color: '#22C55E',
    borderColor: 'rgba(34, 197, 94, 0.3)',
    backgroundColor: 'rgba(34, 197, 94, 0.06)',
  },
  testFailBtn: {
    color: '#EF4444',
    borderColor: 'rgba(239, 68, 68, 0.3)',
    backgroundColor: 'rgba(239, 68, 68, 0.06)',
  },
  empty: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px 20px',
  },
  emptyTitle: {
    fontSize: '14px',
    fontWeight: '600',
    color: '#71717A',
    margin: '0 0 4px',
    fontFamily: "'DM Sans', sans-serif",
  },
  emptyDesc: {
    fontSize: '12px',
    color: '#A1A1AA',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  tabs: {
    display: 'flex',
    gap: '4px',
    marginBottom: '20px',
    backgroundColor: '#F5F5F4',
    padding: '4px',
    borderRadius: '10px',
    width: 'fit-content',
  },
  tab: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    padding: '8px 16px',
    borderRadius: '7px',
    border: 'none',
    backgroundColor: 'transparent',
    fontSize: '13px',
    fontWeight: '500',
    color: '#71717A',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
    transition: 'all 0.15s ease',
  },
  tabActive: {
    backgroundColor: '#FFFFFF',
    color: '#18181B',
    boxShadow: '0 1px 3px rgba(0,0,0,0.08)',
  },
  filterBar: {
    marginBottom: '16px',
  },
  filterInput: {
    padding: '9px 14px',
    borderRadius: '8px',
    border: '1px solid #E7E5E4',
    fontSize: '13px',
    fontFamily: "'DM Sans', sans-serif",
    backgroundColor: '#FFFFFF',
    color: '#18181B',
    outline: 'none',
    width: '300px',
    transition: 'border-color 0.15s ease',
  },
  userCell: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  userAvatar: {
    width: '32px',
    height: '32px',
    borderRadius: '8px',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '13px',
    fontWeight: '600',
    fontFamily: "'Instrument Sans', sans-serif",
    flexShrink: 0,
  },
  userEmail: {
    fontSize: '13px',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  keyName: {
    fontSize: '13px',
    color: '#18181B',
    fontFamily: "'DM Sans', sans-serif",
  },
  dateText: {
    fontSize: '12px',
    color: '#A8A29E',
    fontFamily: "'DM Sans', sans-serif",
  },
};
