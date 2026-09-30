"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const links = [
  { href: "/payroll", label: "TWX Payroll", symbol: "01" },
  { href: "/settlements", label: "Weekly settlements", symbol: "02" },
  { href: "/directory", label: "Directory", symbol: "03" },
];

export function Navigation() {
  const pathname = usePathname();
  return <nav aria-label="Main navigation" className="navigation">{links.map((link) => (
    <Link key={link.href} href={link.href} aria-current={pathname === link.href ? "page" : undefined}>
      <span className="nav-symbol" aria-hidden="true">{link.symbol}</span><span>{link.label}</span>
    </Link>
  ))}</nav>;
}
