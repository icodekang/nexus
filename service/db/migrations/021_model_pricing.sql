-- 021: Multi-dimensional per-model token pricing
CREATE TABLE IF NOT EXISTS model_pricing (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_slug VARCHAR(100) NOT NULL UNIQUE,
    provider_slug VARCHAR(50) NOT NULL,

    prompt_price       DECIMAL(16,12) NOT NULL DEFAULT 0,
    completion_price   DECIMAL(16,12) NOT NULL DEFAULT 0,
    image_price        DECIMAL(16,12),
    reasoning_price    DECIMAL(16,12),
    cache_read_price   DECIMAL(16,12),
    request_price      DECIMAL(16,12),

    pricing_mode VARCHAR(20) NOT NULL DEFAULT 'per_token',
    avg_tokens_per_request INT DEFAULT 5000,

    effective_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    effective_until TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_pricing_model ON model_pricing(model_slug) WHERE is_active = TRUE;
CREATE INDEX idx_pricing_effective ON model_pricing(effective_from);
