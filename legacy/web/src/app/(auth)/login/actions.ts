"use server";

import { redirect } from "next/navigation";
import { revalidatePath } from "next/cache";
import { parseLoginInput, type AuthState } from "@/lib/auth/input";
import { createSupabaseServerClient } from "@/lib/supabase/server";
import { getSupabaseConfig } from "@/lib/supabase/config";

export async function signIn(_previous: AuthState, form: FormData): Promise<AuthState> {
  const input = parseLoginInput(form);
  const rawEmail = form.get("email");
  const email = typeof rawEmail === "string" ? rawEmail.slice(0, 254) : "";
  if (!input) return { error: "Enter a valid email address and password.", email };
  if (!getSupabaseConfig()) return { error: "Sign-in is not configured yet. Contact your administrator.", email };
  try {
    const supabase = await createSupabaseServerClient();
    const { error } = await supabase.auth.signInWithPassword({ email: input.email, password: input.password });
    if (error) return { error: "Unable to sign in. Check your details and try again.", email };
  } catch {
    return { error: "Sign-in is temporarily unavailable. Please try again.", email };
  }
  revalidatePath("/", "layout");
  redirect(input.destination);
}

export async function signOut(): Promise<AuthState> {
  try {
    const supabase = await createSupabaseServerClient();
    const { error } = await supabase.auth.signOut({ scope: "local" });
    if (error) return { error: "Unable to sign out. Please try again." };
  } catch {
    return { error: "Unable to sign out. Please try again." };
  }
  revalidatePath("/", "layout");
  redirect("/login");
}
