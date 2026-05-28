-- 020: User-managed provider API keys (BYOK)
CREATE TABLE IF NOT EXISTS user_provider_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_slug VARCHAR(50) NOT NULL,
    name VARCHAR(100),
    api_key_encrypted TEXT NOT NULL,
    api_key_prefix VARCHAR(24) NOT NULL,
    base_url TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN DEFAULT TRUE,
    priority_level VARCHAR(20) NOT NULL DEFAULT 'prioritized',
    sort_order INT DEFAULT 0,
    always_use BOOLEAN DEFAULT FALSE,
    model_filter TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_upk_user_provider ON user_provider_keys(user_id, provider_slug);
CREATE INDEX idx_upk_priority ON user_provider_keys(user_id, provider_slug, priority_level, sort_order);
