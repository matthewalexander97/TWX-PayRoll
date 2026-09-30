export function EmptyState({ title, description }: { title: string; description: string }) {
  return <div className="empty-state">
    <svg width="44" height="44" viewBox="0 0 44 44" fill="none" aria-hidden="true">
      <rect x="9" y="5" width="26" height="34" rx="5" stroke="currentColor" strokeWidth="1.5" />
      <path d="M16 15h12M16 22h12M16 29h7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
    <h3>{title}</h3><p>{description}</p>
  </div>;
}
