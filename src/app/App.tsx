import { Navigation, areas, type Area } from "../components/Navigation";
import { Settings } from "../features/settings/Settings";
import {
  business,
  emptyReferences,
  errorOf,
  type References,
} from "../contracts/business";
import { recovery } from "../contracts/desktop";
import { ReferenceManager } from "../features/references/ReferenceManager";
import { Settlements } from "../features/settlements/Settlements";
import { Payroll } from "../features/payroll/Payroll";
import { UserManual } from "../features/manual/UserManual";
import { Reports } from "../features/reports/Reports";
import { useEffect, useState } from "react";
import { desktop, type AppInfo, type Theme } from "../contracts/desktop";
export function App() {
  const [references, setReferences] = useState<References>(emptyReferences);
  const [backupConfigured, setBackupConfigured] = useState(true);
  const [dataEpoch, setDataEpoch] = useState(0);
  const [area, setArea] = useState<Area>("TWX Payroll");
  const [menu, setMenu] = useState(false);
  const [theme, setTheme] = useState<Theme>("system");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  async function refreshReferences() {
    setReferences(await business.references());
  }
  function businessChanged() {
    void recovery
      .status()
      .then((s) => setBackupConfigured(!!s.destination && s.automaticEnabled))
      .catch(() => {});
  }
  useEffect(() => {
    if (!info) return;
    let active = true;
    void business
      .references()
      .then((r) => {
        if (active) setReferences(r);
      })
      .catch((c) => {
        if (active)
          setError(
            "Reference records could not be loaded: " + errorOf(c).message,
          );
      });
    void recovery
      .status()
      .then((s) => {
        if (active) setBackupConfigured(!!s.destination && s.automaticEnabled);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [info]);
  useEffect(() => {
    let active = true;
    async function load() {
      if (desktop.available()) {
        try {
          const [preferences, details] = await Promise.all([
            desktop.preferences(),
            desktop.info(),
          ]);
          if (active) {
            setTheme(preferences.theme);
            setInfo(details);
          }
        } catch (cause) {
          if (active)
            setError(
              "Desktop settings could not be loaded. " +
                (typeof cause === "string"
                  ? cause
                  : "Restart the application before continuing."),
            );
        }
      }
      if (active) setReady(true);
    }
    void load();
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      document.documentElement.dataset.theme =
        theme === "system" ? (media.matches ? "dark" : "light") : theme;
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [theme]);
  async function changeTheme(next: Theme) {
    setSaving(true);
    setError("");
    try {
      if (desktop.available()) await desktop.saveTheme(next);
      setTheme(next);
    } catch {
      setError(
        "Theme was not saved. Your previous preference is still active. Please try again.",
      );
    } finally {
      setSaving(false);
    }
  }
  if (!ready)
    return (
      <main className="startup" role="status">
        Opening your workspace…
      </main>
    );
  return (
    <div className="app-shell antialiased">
      <a className="skip-link" href="#main">
        Skip to workspace
      </a>
      <Navigation
        area={area}
        menu={menu}
        onNavigate={(label) => {
          setArea(label);
          setMenu(false);
        }}
      />
      <div className="workspace">
        <header className="topbar">
          <button
            className="menu-button"
            aria-label="Toggle navigation"
            aria-expanded={menu}
            onClick={() => setMenu(!menu)}
          >
            ☰
          </button>
          <span>
            {areas.find(([label]) => label === area)?.[1]}{" "}
            <span className="breadcrumb">/ {area}</span>
          </span>
          <span className="phase-tag">LOCAL WORKSPACE</span>
        </header>
        <main id="main" tabIndex={-1}>
          <div className="page-heading">
            <div>
              <p className="eyebrow">YOUR OPERATIONS, IN ONE PLACE</p>
              <h1>{area}</h1>
              <p className="subtitle">
                {area === "Settings"
                  ? "Appearance, local storage and workspace protection."
                  : area === "Reference Records"
                    ? "Manage TWX codes, plants, jobs and adjustment types."
                    : area === "User Manual"
                      ? "A practical guide to setup, payroll, reports and recovery."
                      : "Manage pay periods, saved records and operations."}
              </p>
            </div>
          </div>
          {!desktop.available() && (
            <div className="notice">
              Browser preview — desktop storage is unavailable. Theme changes in
              this preview last only for this session.
            </div>
          )}
          {error && (
            <div className="error" role="alert">
              {error}
            </div>
          )}
          {!backupConfigured && area !== "Settings" && (
            <div className="notice">
              Automatic backup protection is not configured. Choose a backup
              folder and enable automatic backups in Settings. You can continue
              entering synthetic test records.
            </div>
          )}
          {area === "User Manual" ? (
            <UserManual />
          ) : area === "Settings" ? (
            <>
              <Settings
                theme={theme}
                info={info}
                saving={saving}
                disabled={saving || (desktop.available() && info === null)}
                onChange={changeTheme}
                onRestored={async () => {
                  const [preferences, details] = await Promise.all([
                    desktop.preferences(),
                    desktop.info(),
                  ]);
                  setTheme(preferences.theme);
                  setInfo(details);
                  setError("");
                  await refreshReferences();
                  setDataEpoch((v) => v + 1);
                }}
              />
            </>
          ) : area === "TWX Payroll" ? (
            <Payroll
              key={dataEpoch}
              refs={references}
              enabled={info !== null}
              onChanged={businessChanged}
            />
          ) : area === "Weekly Settlements / Loads" ? (
            <Settlements
              key={dataEpoch}
              refs={references}
              enabled={info !== null}
              onChanged={businessChanged}
            />
          ) : area === "Drivers" ||
            area === "Owners" ||
            area === "Trucks" ||
            area === "Reference Records" ? (
            info ? (
              <ReferenceManager
                key={area}
                refs={references}
                kinds={
                  area === "Drivers"
                    ? ["drivers", "driver_types"]
                    : area === "Owners"
                      ? ["owners"]
                      : area === "Trucks"
                        ? ["trucks"]
                        : ["twx_codes", "plants", "jobs", "adjustment_types"]
                }
                onChanged={refreshReferences}
              />
            ) : (
              <div className="panel business-panel">
                <p>
                  Reference editing is available in the desktop application
                  after storage opens.
                </p>
              </div>
            )
          ) : (
            <Reports
              key={dataEpoch}
              refs={references}
              enabled={info !== null}
            />
          )}
          <footer>
            TWX Payroll{" "}
            <span>Payroll & settlements · Local SQLite workspace</span>
          </footer>
        </main>
      </div>
    </div>
  );
}
