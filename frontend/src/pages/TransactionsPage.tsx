import { useState } from "react";
import { useQuery, useMutation } from "urql";
import { ACCOUNTS_QUERY, BANK_ENTRIES_QUERY, DELETE_BANK_ENTRY } from "../lib/urql";
import type { Account, BankEntry } from "../lib/types";
import DataTable from "../components/shared/DataTable";
import DatePicker from "../components/shared/DatePicker";
import Modal from "../components/shared/Modal";
import BankEntryForm from "../components/BankEntryForm";

function formatCents(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

export default function TransactionsPage() {
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [accountId, setAccountId] = useState<number | undefined>();

  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<BankEntry | null>(null);
  const [viewing, setViewing] = useState<BankEntry | null>(null);

  const [{ data: acctData }] = useQuery({ query: ACCOUNTS_QUERY });
  const accounts = (acctData?.accounts ?? []) as Account[];

  const [{ data, fetching, error }] = useQuery({
    query: BANK_ENTRIES_QUERY,
    variables: { dateFrom: dateFrom || undefined, dateTo: dateTo || undefined, accountId },
  });

  const [, executeDelete] = useMutation(DELETE_BANK_ENTRY);

  const entries: BankEntry[] = data?.bankEntries ?? [];

  const handleDelete = async (entry: BankEntry) => {
    if (!window.confirm(`Delete "${entry.description || entry.date}"?`)) return;
    await executeDelete({ id: entry.id });
    setViewing(null);
    setEditing(null);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold">Transactions</h1>
        <button
          onClick={() => setCreating(true)}
          className="px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700"
        >
          + New Transaction
        </button>
      </div>

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
            render: (r: BankEntry) => (
              <span className={r.amountCents < 0 ? "text-red-600" : "text-green-700"}>
                {formatCents(r.amountCents)}
              </span>
            ),
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
              r.accountEntries.length === 0 ? (
                <span className="text-amber-600 font-medium">Needs allocation</span>
              ) : (
                r.accountEntries
                  .map((ae) => `${ae.account?.name ?? "?"}: ${formatCents(ae.amountCents)}`)
                  .join(", ")
              ),
          },
        ]}
        rows={entries}
        keyExtractor={(r: BankEntry) => r.id}
        onRowClick={(r: BankEntry) => setViewing(r)}
      />

      {creating && (
        <Modal open onClose={() => setCreating(false)} title="New Transaction">
          <BankEntryForm onDone={() => setCreating(false)} onCancel={() => setCreating(false)} />
        </Modal>
      )}

      {viewing && (
        <Modal open onClose={() => setViewing(null)} title="Transaction">
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-500">Date</span>
              <span>{viewing.date}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Amount</span>
              <span className={viewing.amountCents < 0 ? "text-red-600 font-semibold" : "text-green-700 font-semibold"}>
                {formatCents(viewing.amountCents)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Description</span>
              <span>{viewing.description ?? "–"}</span>
            </div>
            {viewing.notes && (
              <div className="flex justify-between">
                <span className="text-gray-500">Notes</span>
                <span>{viewing.notes}</span>
              </div>
            )}
            {viewing.externalId && (
              <div className="flex justify-between">
                <span className="text-gray-500">External ID</span>
                <span>{viewing.externalId}</span>
              </div>
            )}
            <div>
              <div className="text-gray-500 mb-1">Splits</div>
              {viewing.accountEntries.length === 0 && (
                <div className="text-amber-600">No allocation — needs distribution</div>
              )}
              {viewing.accountEntries.map((ae) => (
                <div key={ae.id} className="flex justify-between text-sm">
                  <span>{ae.account?.name ?? "?"}</span>
                  <span>{formatCents(ae.amountCents)}</span>
                </div>
              ))}
            </div>
            <div className="flex justify-end gap-2 pt-2 border-t">
              <button
                onClick={() => handleDelete(viewing)}
                className="px-4 py-2 rounded border text-red-600 hover:bg-red-50"
              >
                Delete
              </button>
              <button
                onClick={() => {
                  setEditing(viewing);
                  setViewing(null);
                }}
                className="px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700"
              >
                Edit
              </button>
            </div>
          </div>
        </Modal>
      )}

      {editing && (
        <Modal open onClose={() => setEditing(null)} title={`Edit Transaction ${editing.id}`}>
          <BankEntryForm
            initial={editing}
            onDone={() => setEditing(null)}
            onCancel={() => setEditing(null)}
          />
        </Modal>
      )}
    </div>
  );
}
