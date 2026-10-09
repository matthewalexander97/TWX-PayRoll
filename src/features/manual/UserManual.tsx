import sections from "./sections.json";
export function UserManual() {
  return (
    <div className="manual-layout">
      <nav
        id="manual-top"
        className="panel manual-index"
        aria-label="Manual sections"
      >
        <div className="manual-index-heading">
          <p className="eyebrow">USER GUIDE</p>
          <h2>Table of contents</h2>
          <p>Choose a topic to jump to its instructions.</p>
        </div>
        <ol className="manual-topics">
          {sections.map((section, index) => (
            <li key={section.id}>
              <a href={`#manual-${section.id}`}>
                <span className="manual-topic-number" aria-hidden="true">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span>{section.title}</span>
              </a>
            </li>
          ))}
        </ol>
      </nav>
      <div className="manual-content">
        {sections.map((section, index) => (
          <section
            key={section.id}
            className="panel manual-section"
            aria-labelledby={`manual-${section.id}`}
          >
            <div className="manual-section-heading">
              <p className="eyebrow">
                SECTION {String(index + 1).padStart(2, "0")}
              </p>
              <h2 id={`manual-${section.id}`} tabIndex={-1}>
                {section.title}
              </h2>
            </div>
            <div className="manual-prose">
              {section.paragraphs.map((paragraph) => (
                <p key={paragraph}>{paragraph}</p>
              ))}
            </div>
            <a
              className="manual-return"
              href="#manual-top"
              onClick={(e) => {
                e.preventDefault();
                document
                  .querySelector<HTMLAnchorElement>(".manual-index a")
                  ?.focus();
              }}
            >
              <span aria-hidden="true">↑</span> Back to section list
            </a>
          </section>
        ))}
      </div>
    </div>
  );
}
