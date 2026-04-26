-- Migration: 012_remove_provider_slug_unique
-- Description: Remove unique constraint from provider_slug to allow multiple keys per provider

ALTER TABLE provider_keys DROP CONSTRAINT IF EXISTS uq_provider_slug;