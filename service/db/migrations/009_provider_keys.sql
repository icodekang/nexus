-- Migration: 009_provider_keys
-- Description: Create provider_keys table for storing encrypted API keys per LLM provider

CREATE TABLE IF NOT EXISTS provider_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_slug VARCHAR(50) NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    api_key_prefix VARCHAR(12) NOT NULL,
    base_url TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN DEFAULT TRUE,
    priority INT DEFAULT 100,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT uq_provider_slug UNIQUE (provider_slug)
);

CREATE INDEX idx_provider_keys_provider_slug ON provider_keys(provider_slug);
CREATE INDEX idx_provider_keys_active ON provider_keys(is_active) WHERE is_active = TRUE;
