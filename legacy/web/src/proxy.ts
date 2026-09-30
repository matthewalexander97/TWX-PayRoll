import type { NextRequest } from "next/server";
import { refreshSession } from "@/lib/supabase/proxy";

export async function proxy(request: NextRequest) {
  return refreshSession(request);
}

export const config = {
  matcher: ["/", "/login", "/payroll/:path*", "/settlements/:path*", "/directory/:path*"],
};
