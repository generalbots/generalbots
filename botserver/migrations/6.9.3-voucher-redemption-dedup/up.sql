-- #2867 — a contact may redeem a voucher only once. The conditional
-- uses_count increment in redeem_voucher closes the max_uses race; this unique
-- index closes the per-user dedup race at the storage layer so two concurrent
-- inserts cannot both succeed.
CREATE UNIQUE INDEX IF NOT EXISTS cloud_voucher_redemptions_voucher_contact_uniq
    ON cloud_voucher_redemptions (voucher_id, contact_id);
