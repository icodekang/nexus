import { ApiError } from '../api/client';
import { useI18n } from '../i18n';

/**
 * Error code to i18n key mapping
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
 * Get translated error message from an ApiError
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
 * Get error with optional contact support message
 */
export function getErrorWithSupport(err: unknown, t: (key: string) => string): string {
  const message = getErrorMessage(err, t);
  // Don't add support message for rate limit errors
  if (err instanceof ApiError && err.code === 'rate_limit_exceeded') {
    return message;
  }
  return `${message} ${t('errors.contact_support')}`;
}
