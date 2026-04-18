/**
 * @file Error Utilities - 错误处理工具
 * 提供 API 错误到国际化消息的映射和转换
 */

/**
 * AdminApiError - API 错误类
 * @description 扩展 Error 类，包含错误码用于 i18n 映射
 */
export class AdminApiError extends Error {
  code: string;
  constructor(message: string, code: string) {
    super(message);
    this.name = 'AdminApiError';
    this.code = code;
  }
}

/**
 * 错误码到 i18n key 的映射表
 * @description 将后端返回的错误码映射到对应的翻译 key
 */
const ERROR_CODE_MAP: Record<string, string> = {
  // Auth errors
  invalid_credentials: 'errors.invalid_credentials',
  unauthorized: 'errors.unauthorized',
  forbidden: 'errors.forbidden',

  // Network errors
  network_error: 'errors.network_error',
  request_failed: 'errors.request_failed',
  internal_error: 'errors.internal_error',

  // CRUD errors
  create_failed: 'errors.create_failed',
  update_failed: 'errors.update_failed',
  delete_failed: 'errors.delete_failed',

  // Browser account errors
  qr_generate_failed: 'errors.qr_generate_failed',
  auth_failed: 'errors.auth_failed',
  account_add_failed: 'errors.account_add_failed',
  account_delete_failed: 'errors.account_delete_failed',
};

/**
 * getErrorMessage - 从错误获取翻译后的错误消息
 * @param err - 错误对象（通常是 AdminApiError 实例）
 * @param t - 翻译函数
 * @returns 翻译后的错误消息
 *
 * 优先级：
 * 1. 从 ERROR_CODE_MAP 查找对应翻译
 * 2. 回退到服务端返回的原始消息
 * 3. 最后使用通用错误消息
 */
export function getErrorMessage(err: unknown, t: (key: string) => string): string {
  if (err instanceof AdminApiError) {
    // Try to get translated message from error code
    const i18nKey = ERROR_CODE_MAP[err.code];
    if (i18nKey) {
      const translated = t(i18nKey);
      if (translated !== i18nKey) {
        return translated;
      }
    }
    // Fallback to server message if available
    if (err.message && err.message.trim()) {
      return err.message;
    }
    // Fallback to generic error
    return t('errors.try_again');
  }
  // Handle plain Error objects
  if (err instanceof Error) {
    if (err.message && err.message.trim()) {
      return err.message;
    }
  }
  return t('errors.try_again');
}

/**
 * getErrorWithSupport - 获取错误消息并附加客服提示
 * @param err - 错误对象
 * @param t - 翻译函数
 * @returns 错误消息（可能附加客服联系方式）
 */
export function getErrorWithSupport(err: unknown, t: (key: string) => string): string {
  const message = getErrorMessage(err, t);
  return `${message} ${t('errors.contact_support')}`;
}

/**
 * isNetworkError - 判断是否为网络错误
 * @param err - 错误对象
 * @returns 是否为网络错误
 */
export function isNetworkError(err: unknown): boolean {
  if (err instanceof AdminApiError) {
    return err.code === 'network_error';
  }
  if (err instanceof Error) {
    const msg = err.message.toLowerCase();
    return msg.includes('failed to fetch') || msg.includes('networkerror') || msg.includes('network error');
  }
  return false;
}
