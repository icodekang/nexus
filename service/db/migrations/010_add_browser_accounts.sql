-- Browser accounts for ZeroToken (QR code authentication)
CREATE TABLE browser_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL,
    email VARCHAR(255),
    session_data_encrypted TEXT NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    request_count BIGINT NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_browser_accounts_provider ON browser_accounts(provider);
CREATE INDEX idx_browser_accounts_status ON browser_accounts(status);

-- QR code sessions for auth flow
CREATE TABLE qr_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES browser_accounts(id) ON DELETE CASCADE,
    code VARCHAR(6) NOT NULL,
    code_expires_at TIMESTAMPTZ NOT NULL,
    auth_completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_qr_sessions_code ON qr_sessions(code);
CREATE INDEX idx_qr_sessions_account_id ON qr_sessions(account_id);
CREATE INDEX idx_qr_sessions_expires ON qr_sessions(code_expires_at) WHERE auth_completed_at IS NULL;
