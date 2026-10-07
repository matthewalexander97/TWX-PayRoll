PRAGMA application_id = 1415010384;
CREATE TABLE backup_settings (
 id INTEGER PRIMARY KEY CHECK(id=1),
 destination TEXT,
 automatic_enabled INTEGER NOT NULL DEFAULT 1 CHECK(automatic_enabled IN (0,1)),
 retention_count INTEGER NOT NULL DEFAULT 10 CHECK(retention_count BETWEEN 2 AND 100),
 change_generation INTEGER NOT NULL DEFAULT 0 CHECK(change_generation>=0),
 saved_generation INTEGER NOT NULL DEFAULT -1,
 last_success_at INTEGER NOT NULL DEFAULT 0,
 last_attempt_at INTEGER NOT NULL DEFAULT 0
) STRICT;
INSERT INTO backup_settings(id) VALUES(1);
CREATE TABLE backup_runs (
 id TEXT PRIMARY KEY NOT NULL,
 started_at INTEGER NOT NULL,
 created_at TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('MANUAL','AUTOMATIC','PRE_RESTORE','RESTORE')),
 path TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('RUNNING','SUCCESS','FAILED','ROLLED_BACK','INTERRUPTED')),
 schema_version INTEGER,
 size_bytes INTEGER,
 sha256 TEXT,
 validation TEXT NOT NULL DEFAULT 'Pending',
 error_summary TEXT
) STRICT;
CREATE INDEX backup_runs_time ON backup_runs(started_at DESC,id);
CREATE TRIGGER backup_dirty_driver_types_insert AFTER INSERT ON driver_types BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_driver_types_update AFTER UPDATE ON driver_types BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_driver_types_delete AFTER DELETE ON driver_types BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_drivers_insert AFTER INSERT ON drivers BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_drivers_update AFTER UPDATE ON drivers BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_drivers_delete AFTER DELETE ON drivers BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_owners_insert AFTER INSERT ON owners BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_owners_update AFTER UPDATE ON owners BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_owners_delete AFTER DELETE ON owners BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_trucks_insert AFTER INSERT ON trucks BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_trucks_update AFTER UPDATE ON trucks BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_trucks_delete AFTER DELETE ON trucks BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_twx_codes_insert AFTER INSERT ON twx_codes BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_twx_codes_update AFTER UPDATE ON twx_codes BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_twx_codes_delete AFTER DELETE ON twx_codes BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_plants_insert AFTER INSERT ON plants BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_plants_update AFTER UPDATE ON plants BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_plants_delete AFTER DELETE ON plants BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_jobs_insert AFTER INSERT ON jobs BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_jobs_update AFTER UPDATE ON jobs BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_jobs_delete AFTER DELETE ON jobs BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_adjustment_types_insert AFTER INSERT ON adjustment_types BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_adjustment_types_update AFTER UPDATE ON adjustment_types BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_adjustment_types_delete AFTER DELETE ON adjustment_types BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_weekly_settlements_insert AFTER INSERT ON weekly_settlements BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_weekly_settlements_update AFTER UPDATE ON weekly_settlements BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_weekly_settlements_delete AFTER DELETE ON weekly_settlements BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_loads_insert AFTER INSERT ON loads BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_loads_update AFTER UPDATE ON loads BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_loads_delete AFTER DELETE ON loads BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_settlement_adjustments_insert AFTER INSERT ON settlement_adjustments BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_settlement_adjustments_update AFTER UPDATE ON settlement_adjustments BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_settlement_adjustments_delete AFTER DELETE ON settlement_adjustments BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_payroll_entries_insert AFTER INSERT ON payroll_entries BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_payroll_entries_update AFTER UPDATE ON payroll_entries BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
CREATE TRIGGER backup_dirty_payroll_entries_delete AFTER DELETE ON payroll_entries BEGIN UPDATE backup_settings SET change_generation=change_generation+1 WHERE id=1; END;
