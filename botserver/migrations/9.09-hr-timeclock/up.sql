DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS hr_clock_entries (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        person_id UUID NOT NULL,
        entry_type VARCHAR(20) NOT NULL,
        timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        latitude NUMERIC(10,7),
        longitude NUMERIC(10,7),
        geofence_id UUID,
        device_id VARCHAR(100),
        ip_address VARCHAR(64),
        notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_hr_clock_person ON hr_clock_entries(person_id);
    CREATE INDEX IF NOT EXISTS idx_hr_clock_bot ON hr_clock_entries(bot_id);
    CREATE INDEX IF NOT EXISTS idx_hr_clock_timestamp ON hr_clock_entries(timestamp);
    CREATE INDEX IF NOT EXISTS idx_hr_clock_type ON hr_clock_entries(entry_type);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating hr_clock_entries table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS hr_work_periods (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        person_id UUID NOT NULL,
        period_start DATE NOT NULL,
        period_end DATE NOT NULL,
        hours_worked NUMERIC(8,2) NOT NULL DEFAULT 0,
        hours_overtime NUMERIC(8,2) NOT NULL DEFAULT 0,
        hours_night NUMERIC(8,2) NOT NULL DEFAULT 0,
        absences INTEGER NOT NULL DEFAULT 0,
        status VARCHAR(30) NOT NULL DEFAULT 'open',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_hr_period_person ON hr_work_periods(person_id);
    CREATE INDEX IF NOT EXISTS idx_hr_period_dates ON hr_work_periods(period_start, period_end);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating hr_work_periods table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS hr_overtime_rules (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        name VARCHAR(100) NOT NULL,
        weekly_threshold_hours NUMERIC(5,2) NOT NULL DEFAULT 44.0,
        daily_threshold_hours NUMERIC(5,2),
        overtime_multiplier NUMERIC(4,2) NOT NULL DEFAULT 1.5,
        night_shift_multiplier NUMERIC(4,2) NOT NULL DEFAULT 1.2,
        holiday_multiplier NUMERIC(4,2) NOT NULL DEFAULT 2.0,
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_hr_ot_bot ON hr_overtime_rules(bot_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating hr_overtime_rules table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS hr_schedules (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        person_id UUID NOT NULL,
        weekday INTEGER NOT NULL,
        start_time TIME NOT NULL,
        end_time TIME NOT NULL,
        break_minutes INTEGER NOT NULL DEFAULT 60,
        effective_from DATE NOT NULL,
        effective_until DATE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_hr_sched_person ON hr_schedules(person_id);
    CREATE INDEX IF NOT EXISTS idx_hr_sched_weekday ON hr_schedules(weekday);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating hr_schedules table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS hr_holidays (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        name VARCHAR(100) NOT NULL,
        holiday_date DATE NOT NULL,
        is_recurring BOOLEAN NOT NULL DEFAULT false,
        region VARCHAR(50),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_hr_holiday_bot ON hr_holidays(bot_id);
    CREATE INDEX IF NOT EXISTS idx_hr_holiday_date ON hr_holidays(holiday_date);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating hr_holidays table: %', SQLERRM;
END $$;
