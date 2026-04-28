ALTER TABLE browser_accounts
  ADD COLUMN IF NOT EXISTS session_expires_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS session_status VARCHAR(20) NOT NULL DEFAULT 'pending';

UPDATE browser_accounts
SET session_status = status
WHERE status IN ('active', 'pending', 'error', 'expired');
