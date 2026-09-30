"use client";

export default function WorkspaceError({ reset }: { reset: () => void }) {
  return <section className="panel empty-state" role="alert"><h1>We couldn’t load this page</h1><p>Please try again. If the problem continues, contact your administrator.</p><button className="button button-primary" onClick={reset}>Try again</button></section>;
}
