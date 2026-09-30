import { Navigation, areas, type Area } from "../components/Navigation";
import { Settings } from "../features/settings/Settings";
import { Placeholder } from "../features/Placeholder";
import { useEffect, useState } from "react";
import { desktop, type AppInfo, type Theme } from "../contracts/desktop";
export function App() {
  const [area, setArea] = useState<Area>("TWX Payroll");
  const [menu, setMenu] = useState(false);
  const [theme, setTheme] = useState<Theme>("system");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
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
          <span className="phase-tag">FOUNDATION PREVIEW</span>
        </header>
        <main id="main" tabIndex={-1}>
          <div className="page-heading">
            <div>
              <p className="eyebrow">YOUR OPERATIONS, IN ONE PLACE</p>
              <h1>{area}</h1>
              <p className="subtitle">
                {area === "Settings"
                  ? "Make this workspace feel like yours."
                  : "A clear workspace for the work ahead."}
              </p>
            </div>
            <span className="offline-badge">
              <span className="status-dot" /> Offline first
            </span>
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
          {area === "Settings" ? (
            <Settings
              theme={theme}
              info={info}
              saving={saving}
              disabled={saving || (desktop.available() && info === null)}
              onChange={changeTheme}
            />
          ) : (
            <Placeholder area={area} />
          )}
          <footer>
            TWX Payroll{" "}
            <span>Desktop Phase 1 · No business records enabled</span>
          </footer>
        </main>
      </div>
    </div>
  );
}
