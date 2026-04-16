/**
 * @file Modal - 通用弹窗组件
 * 支持键盘 ESC 关闭，点击遮罩层关闭
 * 可自定义标题和内容区宽度
 */
import { useEffect, type ReactNode } from 'react';
import { useI18n } from '../i18n';

interface ModalProps {
  open: boolean;         // 控制弹窗显示/隐藏
  onClose: () => void;   // 关闭回调
  title: string;         // 弹窗标题
  children: ReactNode;   // 弹窗内容
  width?: number;        // 弹窗宽度（默认 480px）
}

/**
 * Modal - 通用弹窗组件
 * @description 渲染在页面最顶层的模态弹窗，带 fadeIn + slideUp 动画效果
 */
export default function Modal({ open, onClose, title, children, width = 480 }: ModalProps) {
  const { t } = useI18n();

  // 监听 ESC 键关闭弹窗
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [open, onClose]);

  // 弹窗未打开时返回 null
  if (!open) return null;

  return (
    // 点击遮罩层关闭弹窗，点击内容区阻止冒泡
    <div style={styles.overlay} onClick={onClose}>
      <div
        style={{ ...styles.modal, width: `${width}px` }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* 弹窗头部：标题 + 关闭按钮 */}
        <div style={styles.header}>
          <h2 style={styles.title}>{title}</h2>
          <button style={styles.closeBtn} onClick={onClose} title={t('common.close')}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <div style={styles.body}>{children}</div>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.4)',
    backdropFilter: 'blur(4px)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
    animation: 'fadeIn 0.15s ease',
  },
  modal: {
    backgroundColor: '#FFFFFF',
    borderRadius: '16px',
    boxShadow: '0 20px 60px rgba(0,0,0,0.15)',
    animation: 'slideUp 0.2s ease',
    maxHeight: '80vh',
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '20px 24px 16px',
    borderBottom: '1px solid #F5F5F4',
  },
  title: {
    fontSize: '16px',
    fontWeight: '600',
    color: '#18181B',
    margin: 0,
    fontFamily: "'DM Sans', sans-serif",
  },
  closeBtn: {
    width: '28px',
    height: '28px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '6px',
    cursor: 'pointer',
    color: '#A1A1AA',
    transition: 'all 0.1s ease',
  },
  body: {
    padding: '20px 24px 24px',
    overflowY: 'auto',
  },
};
