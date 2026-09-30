import type { Metadata } from "next";
import { themeScript } from "@/lib/theme";
import "./globals.css";

export const metadata: Metadata = { title: { default: "TWX Payroll", template: "%s | TWX Payroll" }, description: "The bookkeeping workspace for trucking operations." };

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en" suppressHydrationWarning><head><script dangerouslySetInnerHTML={{ __html: themeScript }} /></head><body><a className="skip-link" href="#main-content">Skip to main content</a>{children}</body></html>;
}
