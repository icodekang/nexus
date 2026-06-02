-- Migration: 032_add_api_key_type
-- Description: Add key_type column to api_keys for nexus key protocol type

ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS key_type VARCHAR(20) NOT NULL DEFAULT 'http_messages';
