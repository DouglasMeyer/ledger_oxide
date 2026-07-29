import { useState, useMemo } from "react";
import { useQuery } from "urql";
import { ACCOUNTS_QUERY, BANK_ENTRIES_QUERY } from "../lib/urql";
import DataTable from "../components/shared/DataTable";
import DatePicker from "../components/shared/DatePicker";

interface BankEntry {
  id: number;
  date: string;
  amountCents: number;
  description: string | null;
  notes: string | null;
  accountEntries: { id: number; amountCents: number; account: { id: number; name: string } | null }[];
}

export default function TransactionsPage() {
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [accountId, setAccountId] = useState<number | undefined>();

  const [{ data: acctData }] = useQuery({ query: ACCOUNTS_QUERY });
  const accounts = useMemo(() => (acctData?.accounts ?? []) as { id: number; name: string }[], [acctData]);

  const [{ data, fetching, error }] = useQuery({
    query: BANK_ENTRIES_QUERY,
    variables: { dateFrom: dateFrom || undefined, dateTo: dateTo || undefined, accountId },
  });

  const entries: BankEntry[] = data?.bankEntries ?? [];

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">Transactions</h1>

      <div className="flex gap-3 mb-4 items-end">
        <div>
          <label className="block text-xs text-gray-500 mb-1">From</label>
          <DatePicker value={dateFrom} onChange={setDateFrom} />
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">To</label>
          <DatePicker value={dateTo} onChange={setDateTo} />
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">Account</label>
          <select
            value={accountId ?? ""}
            onChange={(e) => setAccountId(e.target.value ? Number(e.target.value) : undefined)}
            className="border rounded px-2 py-1"
          >
            <option value="">All</option>
            {accounts.map((a) => (
              <option key={a.id} value={a.id}>{a.name}</option>
            ))}
          </select>
        </div>
      </div>

      {fetching && <div className="text-gray-400">Loading…</div>}
      {error && <div className="text-red-500">{error.message}</div>}

      <DataTable
        columns={[
          { key: "date", header: "Date", render: (r: BankEntry) => r.date },
          {
            key: "amount",
            header: "Amount",
            render: (r: BankEntry) => `$${(r.amountCents / 100).toFixed(2)}`,
            className: "text-right",
          },
          {
            key: "description",
            header: "Description",
            render: (r: BankEntry) => r.description ?? "–",
          },
          {
            key: "splits",
            header: "Splits",
            render: (r: BankEntry) =>
              r.accountEntries
                .map((ae) => `${ae.account?.name ?? "?"}: $${(ae.amountCents / 100).toFixed(2)}`)
                .join(", "),
          },
        ]}
        rows={entries}
        keyExtractor={(r: BankEntry) => r.id}
      />
    </div>
  );
}
