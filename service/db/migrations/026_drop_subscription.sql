-- 026: Remove subscription-based billing
ALTER TABLE users DROP COLUMN IF EXISTS subscription_plan;
ALTER TABLE users DROP COLUMN IF EXISTS subscription_start;
ALTER TABLE users DROP COLUMN IF EXISTS subscription_end;

DROP TABLE IF EXISTS subscriptions;
