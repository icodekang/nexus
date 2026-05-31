-- Migration: 031_make_api_key_id_nullable
-- Description: Allow NULL api_key_id for provider key sourced API calls

ALTER TABLE api_logs ALTER COLUMN api_key_id DROP NOT NULL;
