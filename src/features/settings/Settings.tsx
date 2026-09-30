import type { AppInfo, Theme } from "../../contracts/desktop";
export function Settings({
  theme,
  info,
  saving,
  disabled,
  onChange,
}: {
  theme: Theme;
  info: AppInfo | null;
  saving: boolean;
  disabled: boolean;
  onChange: (theme: Theme) => Promise<void>;
}) {
  return (
    <>
      <section className="panel">
        <div className="panel-heading">
          <h2>Appearance</h2>
          <p>
            Follow Windows or choose a theme. Desktop preferences are saved
            locally.
          </p>
        </div>
        <div className="theme-options">
          {(["system", "light", "dark"] as const).map((value) => (
            <button
              key={value}
              aria-label={value}
              aria-pressed={theme === value}
              disabled={disabled}
              className={`theme-choice ${theme === value ? "selected" : ""}`}
              onClick={() => {
                void onChange(value);
              }}
            >
              <span aria-hidden="true">
                {value === "system" ? "◐" : value === "light" ? "☀" : "☾"}
              </span>
              <strong>
                {value[0]?.toUpperCase()}
                {value.slice(1)}
              </strong>
              <small>
                {value === "system" ? "Follow Windows" : `${value} workspace`}
              </small>
            </button>
          ))}
        </div>
        <p role="status" className="settings-status">
          {saving ? "Saving preference…" : `Current preference: ${theme}`}
        </p>
      </section>
      <section className="panel">
        <div className="panel-heading">
          <h2>Application & storage</h2>
          <p>One local workspace, managed by the desktop application.</p>
        </div>
        <dl className="storage-details">
          <dt>Application version</dt>
          <dd>{info?.version ?? "Desktop only"}</dd>
          <dt>Schema version</dt>
          <dd>{info?.schemaVersion ?? "Desktop only"}</dd>
          <dt>SQLite version</dt>
          <dd>{info?.sqliteVersion ?? "Desktop only"}</dd>
          <dt>Database location</dt>
          <dd className="data-path">
            {info?.databasePath ??
              "Available when opened in the desktop application"}
          </dd>
        </dl>
      </section>
    </>
  );
}
