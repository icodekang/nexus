-- Migration: 003_create_providers
-- Description: Create providers table

CREATE TABLE providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(50) UNIQUE NOT NULL,
    logo_url VARCHAR(500),
    is_active BOOLEAN DEFAULT TRUE,
    priority INT DEFAULT 100,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_providers_slug ON providers(slug);
CREATE INDEX idx_providers_active ON providers(is_active);
