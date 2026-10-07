import sections from "./sections.json";
export function UserManual() {
  return (
    <div className="manual-layout">
      <nav
        id="manual-top"
        className="panel manual-index"
        aria-label="Manual sections"
      >
        <h2>In this manual</h2>
        {sections.map((section) => (
          <a key={section.id} href={`#manual-${section.id}`}>
            {section.title}
          </a>
        ))}
      </nav>
      <div className="manual-content">
        {sections.map((section) => (
          <section
            key={section.id}
            className="panel"
            aria-labelledby={`manual-${section.id}`}
          >
            <h2 id={`manual-${section.id}`} tabIndex={-1}>
              {section.title}
            </h2>
            {section.paragraphs.map((paragraph) => (
              <p key={paragraph}>{paragraph}</p>
            ))}
            <a
              href="#manual-top"
              onClick={(e) => {
                e.preventDefault();
                document
                  .querySelector<HTMLAnchorElement>(".manual-index a")
                  ?.focus();
              }}
            >
              Back to section list
            </a>
          </section>
        ))}
      </div>
    </div>
  );
}
