-- Phase 2: exact fixed-point core. 001_settings.sql is unchanged.
CREATE TABLE driver_types (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), name TEXT NOT NULL CHECK(length(trim(name))>0), payout_rate_ppm INTEGER NOT NULL CHECK(payout_rate_ppm BETWEEN 0 AND 1000000), is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE drivers (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), name TEXT NOT NULL CHECK(length(trim(name))>0), driver_type_id TEXT NOT NULL REFERENCES driver_types(id) ON DELETE RESTRICT, is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE owners (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), name TEXT NOT NULL CHECK(length(trim(name))>0), is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE trucks (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), truck_number TEXT NOT NULL UNIQUE CHECK(length(trim(truck_number))>0), owner_id TEXT NOT NULL REFERENCES owners(id) ON DELETE RESTRICT, default_driver_id TEXT REFERENCES drivers(id) ON DELETE RESTRICT, is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE twx_codes (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), code TEXT NOT NULL UNIQUE CHECK(length(trim(code))>0), label TEXT NOT NULL, is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE plants (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), name TEXT NOT NULL CHECK(length(trim(name))>0), is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE jobs (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), name TEXT NOT NULL CHECK(length(trim(name))>0), plant_id TEXT REFERENCES plants(id) ON DELETE RESTRICT, is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE adjustment_types (id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), name TEXT NOT NULL CHECK(length(trim(name))>0), default_direction TEXT NOT NULL CHECK(default_direction IN ('ADD','DEDUCT')), twx_code_id TEXT REFERENCES twx_codes(id) ON DELETE RESTRICT, is_active INTEGER NOT NULL DEFAULT 1 CHECK(is_active IN (0,1)), created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0)) STRICT;

CREATE TABLE weekly_settlements (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), driver_id TEXT NOT NULL REFERENCES drivers(id) ON DELETE RESTRICT, truck_id TEXT NOT NULL REFERENCES trucks(id) ON DELETE RESTRICT, owner_id TEXT NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
 driver_name_snapshot TEXT NOT NULL, truck_number_snapshot TEXT NOT NULL, owner_name_snapshot TEXT NOT NULL,
 driver_type_name_snapshot TEXT NOT NULL, payout_rate_ppm INTEGER NOT NULL CHECK(payout_rate_ppm BETWEEN 0 AND 1000000),
 period_start TEXT NOT NULL CHECK(length(period_start)=10 AND period_start GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]' AND substr(period_start,1,4)>='0001' AND date(period_start,'+0 days') IS period_start),
 period_end TEXT NOT NULL CHECK(length(period_end)=10 AND period_end GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]' AND substr(period_end,1,4)>='0001' AND date(period_end,'+0 days') IS period_end AND period_end>=period_start), notes TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT','FINALIZED')),
 finalized_at TEXT, snapshot_json TEXT CHECK(snapshot_json IS NULL OR json_valid(snapshot_json)), rule_version TEXT, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0),
 gross_route_cents INTEGER CHECK(gross_route_cents>=0), driver_gross_cents INTEGER CHECK(driver_gross_cents>=0), final_payout_cents INTEGER,
 CHECK((status='DRAFT' AND finalized_at IS NULL AND snapshot_json IS NULL AND rule_version IS NULL AND gross_route_cents IS NULL AND driver_gross_cents IS NULL AND final_payout_cents IS NULL)
 OR (status='FINALIZED' AND finalized_at IS NOT NULL AND snapshot_json IS NOT NULL AND rule_version IS NOT NULL AND gross_route_cents IS NOT NULL AND driver_gross_cents IS NOT NULL AND final_payout_cents IS NOT NULL))
) STRICT;

CREATE TABLE loads (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), settlement_id TEXT NOT NULL REFERENCES weekly_settlements(id) ON DELETE RESTRICT, position INTEGER NOT NULL CHECK(position>=0),
 driver_id TEXT NOT NULL REFERENCES drivers(id) ON DELETE RESTRICT, truck_id TEXT NOT NULL REFERENCES trucks(id) ON DELETE RESTRICT, driver_name_snapshot TEXT NOT NULL, truck_number_snapshot TEXT NOT NULL,
 plant_id TEXT REFERENCES plants(id) ON DELETE RESTRICT, job_id TEXT REFERENCES jobs(id) ON DELETE RESTRICT, plant_name_snapshot TEXT, job_name_snapshot TEXT,
 load_number TEXT NOT NULL, bol_number TEXT NOT NULL, po_number TEXT NOT NULL,
 tons_milli INTEGER NOT NULL CHECK(tons_milli>=0), completion_rate_scaled INTEGER NOT NULL CHECK(completion_rate_scaled>=0),
 total_cents INTEGER CHECK(total_cents>=0), UNIQUE(settlement_id,position)
) STRICT;

CREATE TABLE settlement_adjustments (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), settlement_id TEXT NOT NULL REFERENCES weekly_settlements(id) ON DELETE RESTRICT, position INTEGER NOT NULL CHECK(position>=0),
 adjustment_type_id TEXT NOT NULL REFERENCES adjustment_types(id) ON DELETE RESTRICT, type_name_snapshot TEXT NOT NULL,
 twx_code_id TEXT REFERENCES twx_codes(id) ON DELETE RESTRICT, twx_code_snapshot TEXT, twx_label_snapshot TEXT,
 explanation TEXT NOT NULL, direction TEXT NOT NULL CHECK(direction IN ('ADD','DEDUCT')),
 amount_cents INTEGER NOT NULL CHECK(amount_cents>=0), UNIQUE(settlement_id,position)
) STRICT;

CREATE TABLE payroll_entries (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(trim(id)) > 0), entry_kind TEXT NOT NULL CHECK(entry_kind IN ('DRIVER','OWNER')),
 driver_id TEXT REFERENCES drivers(id) ON DELETE RESTRICT, owner_id TEXT REFERENCES owners(id) ON DELETE RESTRICT, truck_id TEXT NOT NULL REFERENCES trucks(id) ON DELETE RESTRICT,
 payee_name_snapshot TEXT NOT NULL, truck_number_snapshot TEXT NOT NULL,
 twx_code_id TEXT REFERENCES twx_codes(id) ON DELETE RESTRICT, twx_code_snapshot TEXT, twx_label_snapshot TEXT,
 settlement_id TEXT UNIQUE REFERENCES weekly_settlements(id) ON DELETE RESTRICT,
 period_start TEXT NOT NULL CHECK(length(period_start)=10 AND period_start GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]' AND substr(period_start,1,4)>='0001' AND date(period_start,'+0 days') IS period_start),
 period_end TEXT NOT NULL CHECK(length(period_end)=10 AND period_end GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]' AND substr(period_end,1,4)>='0001' AND date(period_end,'+0 days') IS period_end AND period_end>=period_start), notes TEXT NOT NULL, amount_cents INTEGER NOT NULL, ach_charge_cents INTEGER CHECK(ach_charge_cents>=0), total_cents INTEGER NOT NULL,
 status TEXT NOT NULL DEFAULT 'DRAFT' CHECK(status IN ('DRAFT','FINALIZED')),
 finalized_at TEXT, snapshot_json TEXT CHECK(snapshot_json IS NULL OR json_valid(snapshot_json)), rule_version TEXT, created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 row_version INTEGER NOT NULL DEFAULT 1 CHECK(row_version > 0),
 CHECK((entry_kind='DRIVER' AND driver_id IS NOT NULL AND owner_id IS NULL AND settlement_id IS NOT NULL AND ach_charge_cents IS NOT NULL)
 OR (entry_kind='OWNER' AND owner_id IS NOT NULL AND driver_id IS NULL AND settlement_id IS NULL AND ach_charge_cents IS NULL AND amount_cents>=0 AND total_cents=amount_cents)),
 CHECK((status='DRAFT' AND finalized_at IS NULL AND snapshot_json IS NULL AND rule_version IS NULL)
 OR (status='FINALIZED' AND finalized_at IS NOT NULL AND snapshot_json IS NOT NULL AND rule_version IS NOT NULL))
) STRICT;

CREATE TABLE audit_events (
 id INTEGER PRIMARY KEY,
 occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 entity_type TEXT NOT NULL CHECK(entity_type IN ('settlement','payroll')),
 entity_id TEXT NOT NULL,
 category TEXT NOT NULL CHECK(category IN ('CREATE','UPDATE','FINALIZE','CORRECTION')),
 reason TEXT NOT NULL CHECK(length(trim(reason))>0),
 before_json TEXT CHECK(before_json IS NULL OR json_valid(before_json)),
 after_json TEXT NOT NULL CHECK(json_valid(after_json))
) STRICT;
CREATE TRIGGER audit_no_replace BEFORE INSERT ON audit_events WHEN EXISTS(SELECT 1 FROM audit_events WHERE id=NEW.id) BEGIN SELECT RAISE(ABORT,'Audit events are append-only'); END;
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit_events BEGIN SELECT RAISE(ABORT,'Audit events are append-only'); END;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit_events BEGIN SELECT RAISE(ABORT,'Audit events are append-only'); END;
CREATE TRIGGER settlement_no_final_delete BEFORE DELETE ON weekly_settlements WHEN OLD.status='FINALIZED' BEGIN SELECT RAISE(ABORT,'Finalized settlements cannot be deleted'); END;
CREATE TRIGGER payroll_no_final_delete BEFORE DELETE ON payroll_entries WHEN OLD.status='FINALIZED' BEGIN SELECT RAISE(ABORT,'Finalized payroll cannot be deleted'); END;
CREATE INDEX settlements_period ON weekly_settlements(period_end,status,id);
CREATE INDEX payroll_period ON payroll_entries(period_end,entry_kind,id);
CREATE INDEX audit_entity ON audit_events(entity_type,entity_id,id);
CREATE INDEX idx_drivers_driver_type_id ON drivers(driver_type_id);
CREATE INDEX idx_trucks_owner_id ON trucks(owner_id);
CREATE INDEX idx_trucks_default_driver_id ON trucks(default_driver_id);
CREATE INDEX idx_jobs_plant_id ON jobs(plant_id);
CREATE INDEX idx_adjustment_types_twx_code_id ON adjustment_types(twx_code_id);
CREATE INDEX idx_weekly_settlements_driver_id ON weekly_settlements(driver_id);
CREATE INDEX idx_weekly_settlements_truck_id ON weekly_settlements(truck_id);
CREATE INDEX idx_weekly_settlements_owner_id ON weekly_settlements(owner_id);
CREATE INDEX idx_loads_driver_id ON loads(driver_id);
CREATE INDEX idx_loads_truck_id ON loads(truck_id);
CREATE INDEX idx_loads_plant_id ON loads(plant_id);
CREATE INDEX idx_loads_job_id ON loads(job_id);
CREATE INDEX idx_settlement_adjustments_adjustment_type_id ON settlement_adjustments(adjustment_type_id);
CREATE INDEX idx_settlement_adjustments_twx_code_id ON settlement_adjustments(twx_code_id);
CREATE INDEX idx_payroll_entries_driver_id ON payroll_entries(driver_id);
CREATE INDEX idx_payroll_entries_owner_id ON payroll_entries(owner_id);
CREATE INDEX idx_payroll_entries_truck_id ON payroll_entries(truck_id);
CREATE INDEX idx_payroll_entries_twx_code_id ON payroll_entries(twx_code_id);
