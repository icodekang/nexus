ALTER TABLE providers ADD COLUMN IF NOT EXISTS api_type VARCHAR(20) DEFAULT 'openai' CHECK (api_type IN ('openai', 'anthropic'));

UPDATE providers SET api_type = 'openai' WHERE api_type IS NULL;