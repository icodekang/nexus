-- Migration: 005_create_usage_logs
-- Description: Create usage logs table for tracking API calls

CREATE TABLE usage_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES providers(id),
    model_id UUID NOT NULL REFERENCES models(id),
    input_tokens INT NOT NULL,
    output_tokens INT NOT NULL,
    cost DECIMAL(10, 6) NOT NULL,
    latency_ms INT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_usage_logs_user_id ON usage_logs(user_id);
CREATE INDEX idx_usage_logs_created_at ON usage_logs(created_at);
CREATE INDEX idx_usage_logs_provider_id ON usage_logs(provider_id);
