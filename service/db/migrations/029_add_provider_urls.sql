-- Migration: 029_add_provider_urls
-- Description: Add openai_api_url and anthropic_api_url columns to providers table

ALTER TABLE providers ADD COLUMN IF NOT EXISTS openai_api_url VARCHAR(500);
ALTER TABLE providers ADD COLUMN IF NOT EXISTS anthropic_api_url VARCHAR(500);

-- Migrate existing data: populate new columns based on api_type
UPDATE providers SET openai_api_url = api_base_url WHERE api_type = 'openai' AND openai_api_url IS NULL;
UPDATE providers SET anthropic_api_url = api_base_url WHERE api_type = 'anthropic' AND anthropic_api_url IS NULL;
