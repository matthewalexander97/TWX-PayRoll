import { requireUser } from "@/server/auth";
import { EmptyState } from "@/components/empty-state";
import { PageHeading } from "@/components/page-heading";

export const metadata = { title: "Weekly settlements" };

export default async function SettlementsPage() {
  await requireUser();
  return <><PageHeading title="Weekly Load / Settlement Detail" description="Your weekly loads, adjustments, and settlement history." /><section className="panel" aria-labelledby="settlement-title"><header className="panel-heading"><h2 id="settlement-title">Weekly settlements</h2><span className="badge">Workspace ready</span></header><EmptyState title="No settlements yet" description="Weekly load entry and settlement history will be available in a future phase." /></section><p className="page-note">Settlement entry and financial calculations are not available yet.</p></>;
}
