import type { Area } from "../components/Navigation";
export function Placeholder({ area }: { area: Area }) {
  return (
    <>
      <section className="panel empty-panel">
        <span className="empty-icon" aria-hidden="true">
          {area === "Reports" ? "▥" : "▤"}
        </span>
        <span className="section-tag">PLANNED WORKSPACE</span>
        <h2>
          {area === "TWX Payroll" ? "Payroll begins here" : `${area} workspace`}
        </h2>
        <p>
          {area === "TWX Payroll"
            ? "Separate driver and owner payroll views will keep weekly records organized."
            : area === "Weekly Settlements / Loads"
              ? "Weekly periods, loads, and adjustments will come together in a dedicated settlement view."
              : area === "Reports"
                ? "Saved statements and report tools will be added in a later phase."
                : `Your ${area.toLowerCase()} directory will be available in a later phase.`}
        </p>
        <div className="empty-note">
          Business workflows are awaiting the next approved phase.
        </div>
      </section>
      <div className="foundation-grid">
        <section className="panel compact-panel">
          <span className="card-kicker">LOCAL BY DESIGN</span>
          <h2>Your desktop workspace</h2>
          <p>
            Designed for offline work with application-managed SQLite storage.
          </p>
        </section>
        <section className="panel compact-panel">
          <span className="card-kicker">READY FOR REVIEW</span>
          <h2>A focused foundation</h2>
          <p>
            Navigation, appearance, and local settings. Financial rules remain
            pending.
          </p>
        </section>
      </div>
    </>
  );
}
