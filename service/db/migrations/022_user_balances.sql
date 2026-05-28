-- 022: User USD credit balances
CREATE TABLE IF NOT EXISTS user_balances (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    balance DECIMAL(16,8) NOT NULL DEFAULT 0,
    total_purchased DECIMAL(16,8) NOT NULL DEFAULT 0,
    total_consumed DECIMAL(16,8) NOT NULL DEFAULT 0,
    auto_topup_threshold DECIMAL(16,8),
    auto_topup_amount DECIMAL(16,8),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
