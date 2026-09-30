import Link from "next/link";

export default function NotFound() {
  return <main id="main-content" className="not-found"><p className="eyebrow">PAGE NOT FOUND</p><h1>Let’s get you back on track.</h1><p>This page does not exist.</p><Link className="button button-primary" href="/">Return to workspace</Link></main>;
}
