-- 025: Add BYOK tracking fields to api_logs
ALTER TABLE api_logs ADD COLUMN IF NOT EXISTS is_free BOOLEAN DEFAULT FALSE;
ALTER TABLE api_logs ADD COLUMN IF NOT EXISTS provider_key_source VARCHAR(20) DEFAULT 'system';
