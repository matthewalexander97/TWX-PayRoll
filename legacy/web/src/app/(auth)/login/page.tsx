import { redirect } from "next/navigation";
import { getVerifiedUser } from "@/server/auth";
import { getSupabaseConfig } from "@/lib/supabase/config";
import { safeDestination } from "@/lib/auth/input";
import { LoginForm } from "@/components/login-form";
import { ThemeToggle } from "@/components/theme-toggle";

export const metadata = { title: "Sign in" };
export const dynamic = "force-dynamic";

export default async function Login({ searchParams }: { searchParams: Promise<{ next?: string }> }) {
  if (await getVerifiedUser()) redirect("/payroll");
  const destination = safeDestination((await searchParams).next);
  return <div className="auth-page">
    <header className="auth-header"><span className="brand"><span className="brand-mark" aria-hidden="true">T</span> TWX <span className="brand-subtle">Payroll</span></span><ThemeToggle /></header>
    <main id="main-content" className="auth-main">
      <section className="auth-intro"><p className="eyebrow">BUILT FOR THE ROAD AHEAD</p><h1>Your week.<br />In one place.</h1><p>A clear workspace for your trucking payroll and weekly settlements.</p><div className="intro-note"><span aria-hidden="true">↗</span><span>Keep the paperwork moving.</span></div></section>
      <section className="auth-card" aria-labelledby="sign-in-title"><p className="eyebrow">WELCOME BACK</p><h2 id="sign-in-title">Sign in</h2><p className="muted">Access your bookkeeping workspace.</p>
        {getSupabaseConfig() ? <LoginForm destination={destination} /> : <div className="notice setup-notice" role="status"><h3>Workspace setup needed</h3><p>Your administrator needs to configure the development Supabase project before sign-in is available.</p><p className="caption">Setup instructions are in the project README.</p></div>}
      </section>
    </main><footer className="auth-footer">TWX Payroll · A simpler bookkeeping workflow</footer>
  </div>;
}
