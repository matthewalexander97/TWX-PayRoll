import { requireUser } from "@/server/auth";
import { EmptyState } from "@/components/empty-state";
import { PageHeading } from "@/components/page-heading";

export const metadata = { title: "Payroll" };

export default async function PayrollPage() {
  await requireUser();
  return <><PageHeading title="TWX Payroll" description="A dedicated place for driver and owner payroll." />
    <div className="payroll-grid">{["Driver", "Owner"].map((kind) => <section className="panel" key={kind} aria-labelledby={`${kind.toLowerCase()}-title`}><header className="panel-heading"><h2 id={`${kind.toLowerCase()}-title`}>{kind} payroll</h2><span className="badge">Workspace ready</span></header><EmptyState title={`No ${kind.toLowerCase()} payroll yet`} description="Record entry and saved history will be available in a future phase." /></section>)}</div>
    <p className="page-note">This foundation provides navigation and account access. Payroll entry is not available yet.</p></>;
}
