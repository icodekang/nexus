-- 028: Fix api_key_prefix column length (was VARCHAR(12), prefix is 15 chars)
ALTER TABLE user_provider_keys ALTER COLUMN api_key_prefix TYPE VARCHAR(24);
