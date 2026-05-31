-- Migration: 030_drop_models_slug
-- Description: Remove slug column from models (pricing sync was pre-applied)

ALTER TABLE models DROP COLUMN IF EXISTS slug;
