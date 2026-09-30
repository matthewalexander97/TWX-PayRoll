"use client";

import { useActionState } from "react";
import { signOut } from "@/app/(auth)/login/actions";

export function SignOutButton() {
  const [state, action, pending] = useActionState(signOut, {});
  return (
    <form action={action} aria-busy={pending}>
      <button className="button button-secondary" disabled={pending}>{pending ? "Signing out…" : "Sign out"}</button>
      {state.error && <p className="notice notice-error" role="alert">{state.error}</p>}
    </form>
  );
}
