-- 023: Per-request token charge records
CREATE TABLE IF NOT EXISTS token_charges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_log_id UUID REFERENCES api_logs(id) ON DELETE SET NULL,
    generation_id UUID NOT NULL,

    key_source VARCHAR(20) NOT NULL DEFAULT 'system',
    user_provider_key_id UUID REFERENCES user_provider_keys(id) ON DELETE SET NULL,
    provider_key_id UUID REFERENCES provider_keys(id) ON DELETE SET NULL,

    model_slug VARCHAR(100) NOT NULL,
    provider_slug VARCHAR(50) NOT NULL,

    input_tokens INT NOT NULL DEFAULT 0,
    output_tokens INT NOT NULL DEFAULT 0,
    reasoning_tokens INT DEFAULT 0,
    image_count INT DEFAULT 0,
    cache_read_tokens INT DEFAULT 0,

    prompt_cost DECIMAL(16,12) NOT NULL DEFAULT 0,
    completion_cost DECIMAL(16,12) NOT NULL DEFAULT 0,
    image_cost DECIMAL(16,12) DEFAULT 0,
    reasoning_cost DECIMAL(16,12) DEFAULT 0,
    cache_read_cost DECIMAL(16,12) DEFAULT 0,
    request_cost DECIMAL(16,12) DEFAULT 0,
    total_cost DECIMAL(16,12) NOT NULL DEFAULT 0,

    is_free BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_charges_user ON token_charges(user_id, created_at DESC);
CREATE INDEX idx_charges_generation ON token_charges(generation_id);
CREATE INDEX idx_charges_key_source ON token_charges(user_id, key_source);
