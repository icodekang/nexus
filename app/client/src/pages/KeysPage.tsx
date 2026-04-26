/**
 * @file KeysPage - API 密钥管理页面
 * 展示用户创建的 API 密钥列表，支持创建、删除、显示/隐藏密钥
 * 新密钥只在创建后显示一次，无法再次查看
 */
import { useState, useEffect, useCallback } from 'react';
import { Key, Plus, Trash2, Copy, Check, Eye, EyeOff, AlertCircle } from 'lucide-react';
import { fetchApiKeys, createApiKey, deleteApiKey, type ApiKey } from '../api/client';
import { useI18n } from '../i18n';
import { getErrorMessage } from '../utils/errors';
import './KeysPage.css';

/**
 * KeysPage - API 密钥管理主组件
 * @description 加载/创建/删除 API 密钥，支持复制和显示/隐藏
 */
export default function KeysPage() {
  const { t, locale } = useI18n();

  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  // 新创建的密钥（只显示一次）
  const [newKey, setNewKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  // 密钥前缀显示/完整显示的状态集合
  const [visibleKeys, setVisibleKeys] = useState<Set<string>>(new Set());
  const [error, setError] = useState('');
  const [toast, setToast] = useState<{ show: boolean; message: string }>({ show: false, message: '' });

  // 加载密钥列表
  const loadKeys = useCallback(async () => {
    try {
      const res = await fetchApiKeys();
      setKeys(res.data);
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadKeys();
  }, [loadKeys]);

  const handleCreate = async () => {
    if (!newKeyName.trim()) return;
    setCreating(true);
    setError('');
    try {
      const res = await createApiKey(newKeyName.trim());
      setNewKey(res.key);
      setNewKeyName('');
      setShowCreate(false);
      loadKeys();
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    } finally {
      setCreating(false);
    }
  };

  // 删除 API 密钥
  const handleDelete = async (keyId: string) => {
    try {
      await deleteApiKey(keyId);
      setKeys((prev) => prev.filter((k) => k.id !== keyId));
    } catch (err: unknown) {
      setError(getErrorMessage(err, t));
    }
  };

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setToast({ show: true, message: t('keys.copiedSuccess') || '复制成功' });
    setTimeout(() => {
      setCopied(false);
      setToast({ show: false, message: '' });
    }, 2000);
  };

  const toggleVisibility = (keyId: string) => {
    setVisibleKeys((prev) => {
      const next = new Set(prev);
      if (next.has(keyId)) next.delete(keyId);
      else next.add(keyId);
      return next;
    });
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString(locale === 'zh' ? 'zh-CN' : 'en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  return (
    <div className="keys-page">
      <header className="keys-header">
        <div className="keys-header-text">
          <h1 className="keys-title">{t('keys.title')}</h1>
          <p className="keys-subtitle">{t('keys.subtitle')}</p>
        </div>
        <button className="keys-create-btn" onClick={() => setShowCreate(!showCreate)}>
          <Plus size={16} />
          {t('keys.newKey')}
        </button>
      </header>

      {error && (
        <div className="keys-error">
          <AlertCircle size={14} />
          {error}
        </div>
      )}

      {/* New key revealed once */}
      {newKey && (
        <div className="keys-new-reveal">
          <div className="keys-new-reveal-header">
            <span className="keys-new-reveal-label">{t('keys.newKeyReveal')}</span>
            <span className="keys-new-reveal-warn">{t('keys.copyWarning')}</span>
          </div>
          <div className="keys-new-reveal-value">
            <code>{newKey}</code>
            <button className="keys-copy-btn" onClick={() => handleCopy(newKey)}>
              {copied ? <Check size={14} /> : <Copy size={14} />}
            </button>
          </div>
          <button className="keys-new-reveal-dismiss" onClick={() => setNewKey(null)}>
            {t('common.dismiss')}
          </button>
        </div>
      )}

      {/* Create form */}
      {showCreate && (
        <div className="keys-create-form">
          <input
            className="keys-create-input"
            placeholder={t('keys.keyNamePlaceholder')}
            value={newKeyName}
            onChange={(e) => setNewKeyName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
            autoFocus
          />
          <div className="keys-create-actions">
            <button className="keys-create-submit" onClick={handleCreate} disabled={creating || !newKeyName.trim()}>
              {creating ? t('common.creating') : t('common.create')}
            </button>
            <button className="keys-create-cancel" onClick={() => setShowCreate(false)}>
              {t('common.cancel')}
            </button>
          </div>
        </div>
      )}

      {/* Keys list */}
      <div className="keys-list">
        {loading ? (
          <div className="keys-loading">{t('keys.loadingKeys')}</div>
        ) : keys.length === 0 ? (
          <div className="keys-empty">
            <Key size={28} strokeWidth={1.5} />
            <h3>{t('keys.noKeys')}</h3>
            <p>{t('keys.noKeysDesc')}</p>
          </div>
        ) : (
          keys.map((key) => (
            <div key={key.id} className="keys-item">
              <div className="keys-item-icon">
                <Key size={16} />
              </div>
              <div className="keys-item-info">
                <div className="keys-item-name">{key.name || t('keys.unnamedKey')}</div>
                <div className="keys-item-meta">
                  <code className="keys-item-prefix">
                    {visibleKeys.has(key.id) ? key.key_prefix : key.key_prefix.slice(0, 8) + '••••••'}
                  </code>
                  <span className="keys-item-date">{t('keys.createdDate', { date: formatDate(key.created_at) })}</span>
                  {key.last_used_at && (
                    <span className="keys-item-date">{t('keys.lastUsed', { date: formatDate(key.last_used_at) })}</span>
                  )}
                </div>
              </div>
              <div className="keys-item-actions">
                <span className={`keys-item-status ${key.is_active ? 'active' : 'inactive'}`}>
                  {key.is_active ? t('common.active') : t('common.inactive')}
                </span>
                <button className="keys-action-btn" onClick={() => toggleVisibility(key.id)} title={t('keys.toggleVisibility')}>
                  {visibleKeys.has(key.id) ? <EyeOff size={14} /> : <Eye size={14} />}
                </button>
                <button className="keys-action-btn keys-delete-btn" onClick={() => handleDelete(key.id)} title={t('keys.deleteKey')}>
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Toast notification */}
      {toast.show && (
        <div className="keys-toast">
          <div className="keys-toast-icon">
            <Check size={16} />
          </div>
          <span className="keys-toast-message">{toast.message}</span>
        </div>
      )}
    </div>
  );
}
