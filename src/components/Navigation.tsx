export const areas = [
  ["TWX Payroll", "Payroll"],
  ["Weekly Settlements / Loads", "Settlements"],
  ["Drivers", "Directory"],
  ["Owners", "Directory"],
  ["Trucks", "Directory"],
  ["Reports", "Reports"],
  ["Settings", "Preferences"],
  ["User Manual", "Help"],
] as const;

export type Area = (typeof areas)[number][0];
export function Navigation({
  area,
  menu,
  onNavigate,
}: {
  area: Area;
  menu: boolean;
  onNavigate: (area: Area) => void;
}) {
  return (
    <aside
      className={`sidebar ${menu ? "is-open" : ""}`}
      aria-label="Application navigation"
    >
      <div className="brand">
        <span className="brand-mark">T</span>
        <div>
          <strong>TWX Payroll</strong>
          <small>OPERATIONS WORKSPACE</small>
        </div>
      </div>
      <div className="nav-label">WORKSPACE</div>
      <nav>
        {areas.map(([label], index) => (
          <button
            key={label}
            className={`nav-item ${area === label ? "active" : ""}`}
            aria-current={area === label ? "page" : undefined}
            onClick={() => {
              onNavigate(label);
            }}
          >
            <span className="nav-symbol" aria-hidden="true">
              {["▤", "▦", "◉", "◇", "▱", "▥", "⚙", "?"][index]}
            </span>
            {label}
          </button>
        ))}
      </nav>
      <div className="sidebar-foot">
        <span className="status-dot" /> Local workspace
        <small>Payroll & settlements</small>
      </div>
    </aside>
  );
}
