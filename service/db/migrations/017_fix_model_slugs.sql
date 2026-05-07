-- Migration: 017_fix_model_slugs
-- Description: Fix model slugs to match code references.
--   This migration is self-contained: it ensures all providers exist first,
--   then fixes/inserts models. The original seed migration (016) is skipped by
--   the incremental migration runner because of its `seed_` name prefix.
--
-- Changes:
--   1. Ensure all 4 base providers exist (idempotent)
--   2. Rename gpt-4 → gpt-4o (code references gpt-4o, not gpt-4)
--   3. Add gpt-4o-mini model
--   4. Add gemini-1-5-pro model

-- ══════════════════════════════════════════════════════════════════════════
-- 1. Ensure all base providers exist (idempotent — ON CONFLICT DO UPDATE)
-- ══════════════════════════════════════════════════════════════════════════
INSERT INTO providers (id, name, slug, logo_url, api_base_url, is_active, priority)
VALUES
    ('a0000000-0000-0000-0000-000000000001', 'DeepSeek',  'deepseek',  NULL, 'https://api.deepseek.com/v1',                       true, 10),
    ('a0000000-0000-0000-0000-000000000002', 'OpenAI',    'openai',    NULL, 'https://api.openai.com/v1',                          true, 20),
    ('a0000000-0000-0000-0000-000000000003', 'Anthropic', 'anthropic', NULL, 'https://api.anthropic.com/v1',                       true, 30),
    ('a0000000-0000-0000-0000-000000000004', 'Google',    'google',    NULL, 'https://generativelanguage.googleapis.com/v1beta',    true, 40)
ON CONFLICT (slug) DO UPDATE SET
    name         = EXCLUDED.name,
    api_base_url = EXCLUDED.api_base_url,
    priority     = EXCLUDED.priority;

-- ══════════════════════════════════════════════════════════════════════════
-- 2. Fix gpt-4 → gpt-4o (only if the old slug exists)
-- ══════════════════════════════════════════════════════════════════════════
UPDATE models
SET name           = 'GPT-4o',
    slug           = 'gpt-4o',
    model_id       = 'gpt-4o',
    context_window = 128000
WHERE slug = 'gpt-4' AND provider_id = 'openai';

-- ══════════════════════════════════════════════════════════════════════════
-- 3. Insert / upsert base models (idempotent)
-- ══════════════════════════════════════════════════════════════════════════
INSERT INTO models (id, provider_id, name, slug, model_id, mode, context_window, capabilities, is_active, created_at)
VALUES
    ('c0000000-0000-0000-0000-000000000001', 'deepseek',  'DeepSeek Chat',      'deepseek-chat',      'deepseek-chat',              'chat', 64000,   '["streaming", "function_calling", "vision"]',      true, NOW()),
    ('c0000000-0000-0000-0000-000000000002', 'deepseek',  'DeepSeek Coder',     'deepseek-coder',     'deepseek-coder',             'chat', 64000,   '["streaming", "function_calling"]',                 true, NOW()),
    ('c0000000-0000-0000-0000-000000000003', 'openai',    'GPT-4o',             'gpt-4o',             'gpt-4o',                     'chat', 128000,  '["streaming", "function_calling", "vision"]',      true, NOW()),
    ('c0000000-0000-0000-0000-000000000004', 'anthropic', 'Claude 3.5 Sonnet',  'claude-3-5-sonnet',  'claude-3-5-sonnet-20240620',  'chat', 200000,  '["streaming", "function_calling", "vision"]',      true, NOW()),
    ('c0000000-0000-0000-0000-000000000005', 'openai',    'GPT-4o Mini',        'gpt-4o-mini',        'gpt-4o-mini',                'chat', 128000,  '["streaming", "function_calling"]',                 true, NOW()),
    ('c0000000-0000-0000-0000-000000000006', 'google',    'Gemini 1.5 Pro',     'gemini-1-5-pro',     'gemini-1.5-pro',             'chat', 1048576, '["streaming", "function_calling", "vision"]',      true, NOW())
ON CONFLICT (slug) DO UPDATE SET
    name           = EXCLUDED.name,
    model_id       = EXCLUDED.model_id,
    context_window = EXCLUDED.context_window,
    capabilities   = EXCLUDED.capabilities,
    is_active      = EXCLUDED.is_active;
