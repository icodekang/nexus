-- Seed admin user (password: admin123)
-- bcrypt hash generated with cost 12
INSERT INTO users (id, email, password_hash, subscription_plan, is_admin, created_at, updated_at)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'admin@nexus.io',
    '$2b$12$4LwTSGbutlOCoP5gF0eB1.Nl2yqRNQXFfIgWGB/YXe/YH8B6FEm5m',
    'enterprise',
    true,
    NOW(),
    NOW()
) ON CONFLICT (email) DO NOTHING;
