import { requireUser } from "@/server/auth";
import { Navigation } from "@/components/navigation";
import { ThemeToggle } from "@/components/theme-toggle";
import { SignOutButton } from "@/components/sign-out-button";

export const dynamic = "force-dynamic";

export default async function WorkspaceLayout({ children }: { children: React.ReactNode }) {
  const user = await requireUser();
  return <div className="workspace">
    <aside className="sidebar"><div className="brand"><span className="brand-mark" aria-hidden="true">T</span> TWX <span className="brand-subtle">Payroll</span></div><p className="sidebar-label">YOUR WORKSPACE</p><Navigation /><p className="sidebar-footer">Trucking operations.<br />Clearly organized.</p></aside>
    <div className="workspace-body"><header className="workspace-header"><div className="account"><span className="account-avatar" aria-hidden="true">{(user.email ?? "U").slice(0, 1).toUpperCase()}</span><div><span className="caption">SIGNED IN</span><p>{user.email ?? "Workspace account"}</p></div></div><div className="header-actions"><ThemeToggle /><SignOutButton /></div></header>
    <main id="main-content" className="workspace-main">{children}</main><footer className="workspace-footer">TWX Payroll <span>Application foundation</span></footer></div>
  </div>;
}
