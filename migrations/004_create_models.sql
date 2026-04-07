-- Migration: 004_create_models
-- Description: Create models table

CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    model_id VARCHAR(100) NOT NULL,
    mode VARCHAR(20) DEFAULT 'chat',
    price_input DECIMAL(10, 6),
    price_output DECIMAL(10, 6),
    context_window INT,
    capabilities JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT TRUE,
    is_hidden BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_models_provider_id ON models(provider_id);
CREATE INDEX idx_models_slug ON models(slug);
CREATE INDEX idx_models_active ON models(is_active);
