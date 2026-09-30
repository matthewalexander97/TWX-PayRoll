import { redirect } from "next/navigation";
import { getVerifiedUser } from "@/server/auth";

export const dynamic = "force-dynamic";

export default async function Home() {
  redirect(await getVerifiedUser() ? "/payroll" : "/login");
}
