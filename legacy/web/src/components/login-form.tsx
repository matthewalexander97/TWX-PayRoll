"use client";

import { useActionState } from "react";
import { signIn } from "@/app/(auth)/login/actions";

export function LoginForm({ destination }: { destination: string }) {
  const [state, action, pending] = useActionState(signIn, {});
  return (
    <form action={action} className="login-form" aria-busy={pending}>
      <input type="hidden" name="next" value={destination} />
      <div className="field">
        <label htmlFor="email">Email address</label>
        <input id="email" name="email" type="email" autoComplete="username" required maxLength={254} defaultValue={state.email} aria-describedby={state.error ? "login-error" : undefined} />
      </div>
      <div className="field">
        <label htmlFor="password">Password</label>
        <input id="password" name="password" type="password" autoComplete="current-password" required maxLength={1024} aria-describedby={state.error ? "login-error" : undefined} />
      </div>
      {state.error && <p id="login-error" className="notice notice-error" role="alert">{state.error}</p>}
      <button className="button button-primary" type="submit" disabled={pending}>{pending ? "Signing in…" : "Sign in to your workspace"}</button>
      <p className="caption">Use the account provided by your administrator.</p>
    </form>
  );
}
