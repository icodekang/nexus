/**
 * @file Error Utilities - 错误处理工具
 * 提供 API 错误到国际化消息的映射和转换
 */

import { ApiError } from '../api/client';
import { useI18n } from '../i18n';

/**
 * 错误码到 i18n key 的映射表
 * @description 将后端返回的错误码映射到对应的翻译 key
 */
const ERROR_CODE_MAP: Record<string, string> = {
  // Auth errors
  invalid_credentials: 'errors.invalid_credentials',
  user_already_exists: 'errors.user_already_exists',
  user_not_found: 'errors.user_not_found',
  unauthorized: 'errors.unauthorized',
  forbidden: 'errors.forbidden',

  // SMS errors
  sms_send_failed: 'errors.sms_send_failed',
  sms_rate_limit_exceeded: 'errors.sms_rate_limit_exceeded',
  invalid_sms_code: 'errors.invalid_sms_code',
  phone_already_registered: 'errors.phone_already_registered',

  // API errors
  network_error: 'errors.network_error',
  network_error_2: 'errors.network_error',
  request_failed: 'errors.request_failed',
  internal_error: 'errors.internal_error',
  invalid_request: 'errors.invalid_request',

  // Subscription & Billing errors
  subscription_expired: 'errors.subscription_expired',
  subscription_failed: 'errors.subscription_failed',
  payment_failed: 'errors.payment_failed',

  // Rate limit
  rate_limit_exceeded: 'errors.rate_limit_exceeded',

  // Model errors
  model_not_found: 'errors.model_not_found',
  provider_error: 'errors.provider_error',
  invalid_api_key: 'errors.invalid_api_key',
};

/**
 * getErrorMessage - 从 ApiError 获取翻译后的错误消息
 * @param err - 错误对象（通常是 ApiError 实例）
 * @param t - 翻译函数
 * @returns 翻译后的错误消息
 *
 * 优先级：
 * 1. 从 ERROR_CODE_MAP 查找对应翻译
 * 2. 回退到服务端返回的原始消息
 * 3. 最后使用通用错误消息
 */
export function getErrorMessage(err: unknown, t: (key: string) => string): string {
  if (err instanceof ApiError) {
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
  return t('errors.try_again');
}

/**
 * getErrorWithSupport - 获取错误消息并附加客服提示
 * @param err - 错误对象
 * @param t - 翻译函数
 * @returns 错误消息（可能附加客服联系方式）
 *
 * 注意：限流错误不会附加客服提示
 */
export function getErrorWithSupport(err: unknown, t: (key: string) => string): string {
  const message = getErrorMessage(err, t);
  // Don't add support message for rate limit errors
  if (err instanceof ApiError && err.code === 'rate_limit_exceeded') {
    return message;
  }
  return `${message} ${t('errors.contact_support')}`;
}
