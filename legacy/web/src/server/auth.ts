import "server-only";
import { redirect } from "next/navigation";
import { createSupabaseServerClient } from "@/lib/supabase/server";
import { getSupabaseConfig } from "@/lib/supabase/config";

export async function getVerifiedUser() {
  if (!getSupabaseConfig()) return null;
  try {
    const supabase = await createSupabaseServerClient();
    const { data, error } = await supabase.auth.getUser();
    return error || data.user?.is_anonymous ? null : data.user;
  } catch {
    return null;
  }
}

export async function requireUser() {
  const user = await getVerifiedUser();
  if (!user) redirect("/login");
  return user;
}
