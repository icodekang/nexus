import { useState, useEffect, useRef } from 'react';
import { useI18n } from '../i18n';
import { QRCodeSVG } from 'qrcode.react';
import { generateQrCode, type BrowserAccount } from '../api/admin';

interface QrCodeModalProps {
  account: BrowserAccount;
  onClose: () => void;
  onSuccess: () => void;
}

export default function QrCodeModal({ account, onClose, onSuccess }: QrCodeModalProps) {
  const { t } = useI18n();
  const [qrData, setQrData] = useState<{ code: string; expires_at: string; auth_url: string } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    // Generate QR code on mount
    generateQrCode(account.id)
      .then((data) => {
        setQrData(data);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message || 'Failed to generate QR code');
        setLoading(false);
      });
  }, [account.id]);

  useEffect(() => {
    if (!qrData) return;

    // Connect to SSE for real-time status updates
    const API_BASE = import.meta.env.VITE_API_BASE || 'http://localhost:8080';
    const sse = new EventSource(`${API_BASE}/admin/accounts/${account.id}/status`);
    eventSourceRef.current = sse;

    sse.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.status === 'active') {
          onSuccess();
        } else if (data.status === 'error') {
          setError('Authentication failed. Please try again.');
          sse.close();
        }
      } catch {
        // Ignore parse errors
      }
    };

    sse.onerror = () => {
      // SSE connection error - keep trying or show error
    };

    return () => {
      sse.close();
    };
  }, [qrData, account.id, onSuccess]);

  const expiresTime = qrData
    ? new Date(qrData.expires_at).toLocaleTimeString()
    : '';

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
        <div style={styles.header}>
          <h2 style={styles.title}>{t('qrModal.title')}</h2>
          <button style={styles.closeBtn} onClick={onClose}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div style={styles.body}>
          {loading && (
            <div style={styles.loading}>
              <div style={styles.spinner} />
              <p>{t('qrModal.generating')}</p>
            </div>
          )}

          {error && (
            <div style={styles.error}>
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              <p>{error}</p>
            </div>
          )}

          {qrData && !loading && !error && (
            <>
              <div style={styles.providerBadge}>
                {account.provider === 'claude' ? (
                  <span style={{ ...styles.badgeText, color: '#D97706' }}>Claude.ai</span>
                ) : (
                  <span style={{ ...styles.badgeText, color: '#10A37F' }}>ChatGPT</span>
                )}
              </div>

              <div style={styles.qrContainer}>
                {qrData.auth_url ? (
                  <QRCodeSVG
                    value={qrData.auth_url}
                    size={200}
                    level="M"
                    includeMargin={true}
                    style={styles.qrImage}
                  />
                ) : (
                  <div style={styles.qrPlaceholder}>
                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#A1A1AA" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                      <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" />
                      <rect x="14" y="14" width="7" height="7" /><rect x="3" y="14" width="7" height="7" />
                    </svg>
                  </div>
                )}
              </div>

              <div style={styles.instructions}>
                <p style={styles.step}>
                  <span style={styles.stepNum}>1</span>
                  {t('qrModal.step1')}
                </p>
                <div style={styles.urlBox}>
                  <a href={qrData.auth_url} target="_blank" rel="noopener noreferrer" style={styles.authUrl}>
                    {qrData.auth_url}
                  </a>
                </div>
                <p style={styles.orText}>{t('qrModal.or')}</p>
                <p style={styles.step}>
                  <span style={styles.stepNum}>2</span>
                  {t('qrModal.step2')} <strong style={styles.codeHighlight}>{qrData.code}</strong>
                </p>
                <p style={styles.expiry}>{t('qrModal.expiresAt', { time: expiresTime })}</p>
              </div>

              <div style={styles.waiting}>
                <div style={styles.waitingDot} />
                <span>{t('qrModal.waitingAuth')}</span>
              </div>
            </>
          )}
        </div>
      </div>

      <style>{`
        @keyframes spin {
          100% { transform: rotate(360deg); }
        }
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    backgroundColor: '#FFFFFF',
    borderRadius: '16px',
    width: '100%',
    maxWidth: '400px',
    boxShadow: '0 20px 40px rgba(0, 0, 0, 0.15)',
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '20px 24px',
    borderBottom: '1px solid #F5F5F4',
  },
  title: {
    fontSize: '16px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
  },
  closeBtn: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '32px',
    height: '32px',
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '8px',
    cursor: 'pointer',
    color: '#71717A',
  },
  body: {
    padding: '24px',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '16px',
  },
  loading: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '12px',
    padding: '32px',
  },
  spinner: {
    width: '32px',
    height: '32px',
    border: '3px solid #F5F5F4',
    borderTopColor: '#6366F1',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  error: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '8px',
    padding: '16px',
    color: '#EF4444',
  },
  providerBadge: {
    padding: '4px 12px',
    borderRadius: '9999px',
    backgroundColor: '#F5F5F4',
  },
  badgeText: {
    fontSize: '12px',
    fontWeight: '600',
    fontFamily: "'DM Sans', sans-serif",
  },
  qrContainer: {
    padding: '16px',
    backgroundColor: '#FFFFFF',
    borderRadius: '12px',
    border: '1px solid #F5F5F4',
  },
  qrImage: {
    width: '200px',
    height: '200px',
    display: 'block',
  },
  qrPlaceholder: {
    width: '200px',
    height: '200px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#F5F5F4',
    borderRadius: '8px',
  },
  instructions: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '8px',
    textAlign: 'center',
    width: '100%',
  },
  step: {
    fontSize: '13px',
    color: '#71717A',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  stepNum: {
    width: '20px',
    height: '20px',
    borderRadius: '50%',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    fontSize: '11px',
    fontWeight: '600',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
  },
  urlBox: {
    width: '100%',
    padding: '10px 12px',
    backgroundColor: '#F5F5F4',
    borderRadius: '8px',
    overflow: 'hidden',
  },
  authUrl: {
    fontSize: '11px',
    color: '#6366F1',
    fontFamily: "'DM Sans', sans-serif",
    wordBreak: 'break-all',
    textDecoration: 'none',
  },
  orText: {
    fontSize: '12px',
    color: '#A1A1AA',
    margin: '4px 0',
    fontFamily: "'DM Sans', sans-serif",
  },
  codeHighlight: {
    fontFamily: "'DM Sans', monospace",
    fontSize: '14px',
    color: '#18181B',
    letterSpacing: '0.1em',
  },
  expiry: {
    fontSize: '11px',
    color: '#A1A1AA',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  waiting: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '10px 16px',
    backgroundColor: 'rgba(99, 102, 241, 0.08)',
    borderRadius: '8px',
    color: '#6366F1',
    fontSize: '12px',
    fontFamily: "'DM Sans', sans-serif",
  },
  waitingDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    backgroundColor: '#6366F1',
    animation: 'pulse 1.5s ease-in-out infinite',
  },
};
