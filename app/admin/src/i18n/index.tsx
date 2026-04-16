/**
 * @file Admin i18n - 国际化支持
 * 提供多语言切换功能，支持英语和中文
 * 使用 Context 模式提供翻译函数
 */

import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import { en } from './en';
import { zh } from './zh';

// 语言类型
export type Locale = 'en' | 'zh';

// 翻译文件类型
type Translations = typeof en;

// 翻译资源映射
const translations: Record<Locale, Translations> = { en, zh };

/**
 * resolveKey - 解析嵌套的翻译 key
 * @param obj - 翻译对象
 * @param key - 点分隔的 key（如 'common.save'）
 * @returns 翻译文本或 undefined
 */
function resolveKey(obj: any, key: string): string | undefined {
  const parts = key.split('.');
  let current = obj;
  for (const part of parts) {
    if (current == null || typeof current !== 'object') return undefined;
    current = current[part];
  }
  return typeof current === 'string' ? current : undefined;
}

/**
 * detectLocale - 检测用户语言偏好
 * @description 优先级：localStorage > 浏览器语言 > 默认英语
 */
export function detectLocale(): Locale {
  try {
    const saved = localStorage.getItem('nexus_locale');
    if (saved === 'en' || saved === 'zh') return saved;
  } catch {}
  if (typeof navigator !== 'undefined' && navigator.language?.startsWith('zh')) return 'zh';
  return 'en';
}

/**
 * I18nContextValue - i18n Context 值类型
 * @description 提供翻译函数和语言切换功能
 */
interface I18nContextValue {
  t: (key: string, params?: Record<string, string | number>) => string;  // 翻译函数，支持参数替换
  locale: Locale;                  // 当前语言
  setLocale: (locale: Locale) => void;  // 切换语言
}

const I18nContext = createContext<I18nContextValue | null>(null);

/**
 * I18nProvider - i18n 提供者组件
 * @description 包裹应用，提供多语言支持
 */
export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(detectLocale);

  const setLocale = useCallback((l: Locale) => {
    setLocaleState(l);
    try { localStorage.setItem('nexus_locale', l); } catch {}
  }, []);

  const t = useCallback((key: string, params?: Record<string, string | number>): string => {
    let value = resolveKey(translations[locale], key);
    if (value == null) value = resolveKey(translations.en, key);
    if (value == null) return key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        value = value.replaceAll(`{{${k}}}`, String(v));
      }
    }
    return value;
  }, [locale]);

  return <I18nContext.Provider value={{ t, locale, setLocale }}>{children}</I18nContext.Provider>;
}

/**
 * useI18n - 使用 i18n 的 Hook
 * @returns I18nContextValue
 * @throws 如果在 I18nProvider 外使用，抛出错误
 */
export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error('useI18n must be used within I18nProvider');
  return ctx;
}
