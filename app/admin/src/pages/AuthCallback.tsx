/**
 * @file AuthCallback - 浏览器认证回调页面
 * 处理浏览器自动化登录完成后的回调，完成会话激活
 * 展示认证状态（加载中/成功/失败）
 */
import { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { completeBrowserAuth } from '../api/admin';

/**
 * AuthCallback - 认证回调主组件
 * @description 从 URL 获取 code 和 session_id，调用后端完成认证
 */
export default function AuthCallback() {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<'loading' | 'success' | 'error'>('loading');
  const [error, setError] = useState('');

  useEffect(() => {
    // 从 URL 参数获取认证码和会话 ID
    const code = searchParams.get('code');
    const sessionId = searchParams.get('session_id');

    if (!code || !sessionId) {
      setStatus('error');
      setError('Missing required parameters');
      return;
    }

    // 会话数据从 URL 参数中获取（通常来自 cookies 或 postMessage）
    const sessionData = searchParams.get('session_data') || '';

    // 调用后端完成浏览器认证
    completeBrowserAuth(code, sessionId, sessionData)
      .then(() => {
        setStatus('success');
      })
      .catch((err) => {
        setStatus('error');
        setError(err.message || 'Authentication failed');
      });
  }, [searchParams]);

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        {/* 加载中状态 */}
        {status === 'loading' && (
          <>
            <div style={styles.spinner} />
            <h2 style={styles.title}>Completing authentication...</h2>
            <p style={styles.desc}>Please wait while we verify your session.</p>
          </>
        )}

        {/* 认证成功状态 */}
        {status === 'success' && (
          <>
            <div style={styles.iconWrapper}>
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#22C55E" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                <polyline points="22 4 12 14.01 9 11.01" />
              </svg>
            </div>
            <h2 style={{ ...styles.title, color: '#22C55E' }}>Authentication Successful!</h2>
            <p style={styles.desc}>You can close this page and return to the admin panel.</p>
          </>
        )}

        {/* 认证失败状态 */}
        {status === 'error' && (
          <>
            <div style={styles.iconWrapper}>
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#EF4444" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
            </div>
            <h2 style={{ ...styles.title, color: '#EF4444' }}>Authentication Failed</h2>
            <p style={styles.desc}>{error}</p>
            <button style={styles.closeBtn} onClick={() => window.close()}>
              Close
            </button>
          </>
        )}
      </div>

      {/* CSS 动画：旋转动画 */}
      <style>{`
        @keyframes spin {
          100% { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: '100vh',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FAFAF9',
    padding: '20px',
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: '16px',
    padding: '48px 40px',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.08)',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '16px',
    maxWidth: '400px',
    width: '100%',
    textAlign: 'center',
  },
  spinner: {
    width: '48px',
    height: '48px',
    border: '3px solid #F5F5F4',
    borderTopColor: '#6366F1',
    borderRadius: '50%',
    animation: 'spin 1s linear infinite',
  },
  iconWrapper: {
    marginBottom: '8px',
  },
  title: {
    fontSize: '20px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'Instrument Sans', sans-serif",
  },
  desc: {
    fontSize: '14px',
    color: '#71717A',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
    lineHeight: 1.5,
  },
  closeBtn: {
    marginTop: '8px',
    padding: '10px 24px',
    backgroundColor: '#6366F1',
    color: '#FFFFFF',
    border: 'none',
    borderRadius: '8px',
    fontSize: '14px',
    fontWeight: '500',
    cursor: 'pointer',
    fontFamily: "'DM Sans', sans-serif",
  },
};
