-- Migration: 004_create_models
-- Description: Create models table

CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id VARCHAR(50) NOT NULL REFERENCES providers(slug) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    model_id VARCHAR(100) NOT NULL,
    mode VARCHAR(20) DEFAULT 'chat',
    context_window INT NOT NULL,
    capabilities JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_models_provider_id ON models(provider_id);
CREATE INDEX idx_models_slug ON models(slug);
CREATE INDEX idx_models_active ON models(is_active);
