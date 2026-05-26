-- Migration: 018_add_model_description
-- Description: Add description field to models table

ALTER TABLE models ADD COLUMN description TEXT;
