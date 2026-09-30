import { createServerClient } from "@supabase/ssr";
import { NextResponse, type NextRequest } from "next/server";
import { getSupabaseConfig } from "./config";

export async function refreshSession(request: NextRequest) {
  const config = getSupabaseConfig();
  const responseHeaders = { "Cache-Control": "private, no-store" };
  if (!config) return NextResponse.next({ headers: responseHeaders });
  let response = NextResponse.next({ request, headers: responseHeaders });
  const supabase = createServerClient(config.url, config.key, {
    cookies: {
      getAll: () => request.cookies.getAll(),
      setAll(values) {
        values.forEach(({ name, value }) => request.cookies.set(name, value));
        response = NextResponse.next({ request, headers: responseHeaders });
        values.forEach(({ name, value, options }) => response.cookies.set(name, value, options));
      },
    },
  });
  try {
    await supabase.auth.getClaims();
  } catch {
    // Protected layouts verify the user independently and fail closed.
  }
  return response;
}
