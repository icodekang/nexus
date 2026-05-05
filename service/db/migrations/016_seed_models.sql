-- Seed providers for testing
INSERT INTO providers (id, name, slug, logo_url, api_base_url, is_active, priority)
VALUES
    ('a0000000-0000-0000-0000-000000000001', 'DeepSeek', 'deepseek', NULL, 'https://api.deepseek.com/v1', true, 10),
    ('a0000000-0000-0000-0000-000000000002', 'OpenAI', 'openai', NULL, 'https://api.openai.com/v1', true, 20),
    ('a0000000-0000-0000-0000-000000000003', 'Anthropic', 'anthropic', NULL, 'https://api.anthropic.com/v1', true, 30)
ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    api_base_url = EXCLUDED.api_base_url,
    priority = EXCLUDED.priority;

-- Seed models for testing

INSERT INTO models (id, provider_id, name, slug, model_id, mode, context_window, capabilities, is_active, created_at)
VALUES
    ('c0000000-0000-0000-0000-000000000001', 'deepseek', 'DeepSeek Chat', 'deepseek-chat', 'deepseek-chat', 'chat', 64000, '["streaming", "function_calling", "vision"]', true, NOW()),
    ('c0000000-0000-0000-0000-000000000002', 'deepseek', 'DeepSeek Coder', 'deepseek-coder', 'deepseek-coder', 'chat', 64000, '["streaming", "function_calling"]', true, NOW()),
    ('c0000000-0000-0000-0000-000000000003', 'openai', 'GPT-4', 'gpt-4', 'gpt-4', 'chat', 8192, '["streaming", "function_calling", "vision"]', true, NOW()),
    ('c0000000-0000-0000-0000-000000000004', 'anthropic', 'Claude 3.5 Sonnet', 'claude-3-5-sonnet', 'claude-3-5-sonnet-20240620', 'chat', 200000, '["streaming", "function_calling", "vision"]', true, NOW())
ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    model_id = EXCLUDED.model_id,
    context_window = EXCLUDED.context_window,
    capabilities = EXCLUDED.capabilities,
    is_active = EXCLUDED.is_active;