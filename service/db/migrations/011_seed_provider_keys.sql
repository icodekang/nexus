-- Seed provider keys (for testing provider keys admin UI)
-- The api_key_encrypted values were generated using AES-256-GCM encryption
-- with API_KEY_ENCRYPTION_KEY=67624753dd089e39692a86953633a1c73d3d63e749c56e81285a9653d7004cc1
-- Decrypted values:
--   openai:   sk-test-openai-1234567890
--   anthropic: sk-ant-test-anthropic-abcdefghij
--   deepseek: sk-test-deepseek-abcdefghijklmnop

INSERT INTO provider_keys (id, provider_slug, api_key_encrypted, api_key_prefix, base_url, is_active, priority, created_at, updated_at)
VALUES
    ('b0000000-0000-0000-0000-000000000001', 'openai', 'tXblX+0doXocLpOGQSYWiKxesuVMvN3DrUpPxXF96qUy88TEaneBfrO+zBcMnnBgGZO8FtA=', 'sk-test-open', 'https://api.openai.com/v1', true, 1, NOW(), NOW()),
    ('b0000000-0000-0000-0000-000000000002', 'anthropic', 'XhTrUQzMv1gYTalenoGL3PFWIiJ0A88jODPz3u1tBngI4oS/oiD9XzTzwtQPyXyp6c8B3m6+LtrGxBLq', 'sk-ant-test-', 'https://api.anthropic.com/v1', true, 2, NOW(), NOW()),
    ('b0000000-0000-0000-0000-000000000003', 'deepseek', 'Kndik/kwx0RD8pA8K/sa0zw6PTyzi1qV+41qJgQi2egxWfInitWAs52S+LHOMjM7wYTVSxB1YCzWiW+mNw==', 'sk-test-deep', 'https://api.deepseek.com/v1', true, 3, NOW(), NOW())
ON CONFLICT (provider_slug) DO UPDATE SET
    api_key_encrypted = EXCLUDED.api_key_encrypted,
    api_key_prefix = EXCLUDED.api_key_prefix,
    base_url = EXCLUDED.base_url,
    is_active = EXCLUDED.is_active,
    priority = EXCLUDED.priority,
    updated_at = NOW();