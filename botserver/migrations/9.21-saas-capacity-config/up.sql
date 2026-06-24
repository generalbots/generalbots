CREATE TABLE IF NOT EXISTS saas_capacity_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cpu_pause_threshold REAL NOT NULL DEFAULT 80.0,
    ram_pause_threshold REAL NOT NULL DEFAULT 85.0,
    disk_pause_threshold REAL NOT NULL DEFAULT 90.0,
    ram_per_free_account_mb REAL NOT NULL DEFAULT 128.0,
    ram_per_shared_account_mb REAL NOT NULL DEFAULT 512.0,
    free_ram_allocation_fraction REAL NOT NULL DEFAULT 0.70,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
INSERT INTO saas_capacity_config DEFAULT VALUES;
