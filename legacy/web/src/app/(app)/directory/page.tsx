import { requireUser } from "@/server/auth";
import { EmptyState } from "@/components/empty-state";
import { PageHeading } from "@/components/page-heading";

export const metadata = { title: "Directory" };

export default async function DirectoryPage() {
  await requireUser();
  return <><PageHeading title="Directory" description="A home for your drivers, owners, trucks, and reference information." /><section className="panel"><EmptyState title="Your directory starts here" description="Reference management will be available after the database design is approved and implemented." /></section></>;
}
