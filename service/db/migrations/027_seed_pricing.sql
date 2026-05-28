-- 027: Seed initial model pricing data (prices per token in USD)
INSERT INTO model_pricing (model_slug, provider_slug, prompt_price, completion_price) VALUES
    ('gpt-4o', 'openai', 0.0000025, 0.000010),
    ('gpt-4o-mini', 'openai', 0.00000015, 0.0000006),
    ('gpt-4-turbo', 'openai', 0.000010, 0.000030),
    ('gpt-3.5-turbo', 'openai', 0.0000005, 0.0000015),
    ('claude-3-5-sonnet', 'anthropic', 0.000003, 0.000015),
    ('claude-3-opus', 'anthropic', 0.000015, 0.000075),
    ('claude-3-haiku', 'anthropic', 0.00000025, 0.00000125),
    ('deepseek-chat', 'deepseek', 0.00000027, 0.0000011),
    ('deepseek-coder', 'deepseek', 0.00000014, 0.00000028),
    ('gemini-1-5-pro', 'google', 0.00000125, 0.000005),
    ('gemini-1-5-flash', 'google', 0.000000075, 0.0000003),
    ('gemini-1-0-pro', 'google', 0.0000005, 0.0000015)
ON CONFLICT (model_slug) DO NOTHING;

INSERT INTO token_packages (name, credits, price, bonus_credits, sort_order) VALUES
    ('入门包', 1.00, 1.00, 0, 1),
    ('标准包', 10.00, 10.00, 0, 2),
    ('专业包', 50.00, 50.00, 5.00, 3),
    ('企业包', 200.00, 200.00, 40.00, 4);
