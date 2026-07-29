import { useQuery } from "urql";
import { ACCOUNTS_QUERY } from "../lib/urql";
import DataTable from "../components/shared/DataTable";

interface Account {
  id: number;
  name: string;
  balanceCents: number;
  active: boolean;
  asset: boolean | null;
  category: string | null;
  position: number | null;
}

export default function AccountsPage() {
  const [{ data, fetching, error }] = useQuery({
    query: ACCOUNTS_QUERY,
    variables: { active: true },
  });

  if (fetching) return <div className="text-gray-400">Loading…</div>;
  if (error) return <div className="text-red-500">{error.message}</div>;

  const accounts: Account[] = data?.accounts ?? [];

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">Accounts</h1>
      <DataTable
        columns={[
          { key: "name", header: "Name", render: (r: Account) => r.name },
          {
            key: "balance",
            header: "Balance",
            render: (r: Account) => `$${(r.balanceCents / 100).toFixed(2)}`,
            className: "text-right",
          },
          {
            key: "type",
            header: "Type",
            render: (r: Account) =>
              r.asset === null ? "–" : r.asset ? "Asset" : "Liability",
          },
          { key: "category", header: "Category", render: (r: Account) => r.category ?? "–" },
        ]}
        rows={accounts}
        keyExtractor={(r: Account) => r.id}
      />
    </div>
  );
}
