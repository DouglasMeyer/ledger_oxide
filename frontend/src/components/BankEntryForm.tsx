import { useState } from "react";
import { useQuery, useMutation } from "urql";
import { ACCOUNTS_QUERY, CREATE_BANK_ENTRY, UPDATE_BANK_ENTRY } from "../lib/urql";
import type { Account, BankEntry } from "../lib/types";
import CurrencyInput from "./shared/CurrencyInput";
import DatePicker from "./shared/DatePicker";
import SplitRowsWidget, { SplitRow } from "./shared/SplitRowsWidget";

interface BankEntryFormProps {
  initial?: BankEntry;
  onDone: () => void;
  onCancel: () => void;
}

function toSplits(entry: BankEntry | undefined): SplitRow[] {
  return entry
    ? entry.accountEntries.map((ae) => ({
        key: `ae-${ae.id}`,
        id: ae.id,
        accountName: ae.account?.name ?? "",
        amountCents: ae.amountCents,
      }))
    : [];
}

export default function BankEntryForm({ initial, onDone, onCancel }: BankEntryFormProps) {
  const isEdit = !!initial;

  const [date, setDate] = useState(initial?.date ?? new Date().toISOString().slice(0, 10));
  const [amountCents, setAmountCents] = useState(initial?.amountCents ?? 0);
  const [description, setDescription] = useState(initial?.description ?? "");
  const [notes, setNotes] = useState(initial?.notes ?? "");
  const [splits, setSplits] = useState<SplitRow[]>(() => toSplits(initial));
  const [error, setError] = useState<string | null>(null);

  const [{ data: acctData }] = useQuery({ query: ACCOUNTS_QUERY });
  const accounts = (acctData?.accounts ?? []) as Account[];

  const [, executeCreate] = useMutation(CREATE_BANK_ENTRY);
  const [, executeUpdate] = useMutation(UPDATE_BANK_ENTRY);

  const handleSubmit = async () => {
    setError(null);

    if (!date) {
      setError("Date is required");
      return;
    }
    if (amountCents === 0) {
      setError("Amount must be non-zero");
      return;
    }

    const cleanSplits = splits.filter((s) => s.accountName.trim());

    const result = isEdit
      ? await executeUpdate({
          id: initial!.id,
          input: {
            date,
            amountCents,
            description: description.trim() || null,
            notes: notes.trim() || null,
            accountEntries: [
              ...toSplits(initial!)
                .filter((s) => !splits.some((c) => c.key === s.key))
                .map((s) => ({ id: s.id, destroy: true })),
              ...cleanSplits.map((s) =>
                s.id
                  ? { id: s.id, accountName: s.accountName.trim(), amountCents: s.amountCents }
                  : { accountName: s.accountName.trim(), amountCents: s.amountCents },
              ),
            ],
          },
        })
      : await executeCreate({
          input: {
            date,
            amountCents,
            description: description.trim() || null,
            notes: notes.trim() || null,
            accountEntries: cleanSplits.map((s) => ({
              accountName: s.accountName.trim(),
              amountCents: s.amountCents,
            })),
          },
        });

    if (result.error) {
      setError(result.error.message);
      return;
    }

    onDone();
  };

  return (
    <div className="space-y-4">
      {error && <div className="p-3 bg-red-50 text-red-700 rounded">{error}</div>}

      <div className="flex gap-3">
        <div>
          <label className="block text-xs text-gray-500 mb-1">Date</label>
          <DatePicker value={date} onChange={setDate} />
        </div>
        <div>
          <label className="block text-xs text-gray-500 mb-1">Amount</label>
          <CurrencyInput value={amountCents} onChange={setAmountCents} />
        </div>
      </div>

      <div>
        <label className="block text-xs text-gray-500 mb-1">Description</label>
        <input
          type="text"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className="border rounded px-2 py-1 w-full"
        />
      </div>

      <div>
        <label className="block text-xs text-gray-500 mb-1">Notes</label>
        <textarea
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          rows={2}
          className="border rounded px-2 py-1 w-full"
        />
      </div>

      <div>
        <label className="block text-xs text-gray-500 mb-1">Split</label>
        <SplitRowsWidget
          accounts={accounts}
          rows={splits}
          totalCents={amountCents}
          onChange={setSplits}
        />
      </div>

      <div className="flex justify-end gap-2 pt-2">
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 rounded border hover:bg-gray-50"
        >
          Cancel
        </button>
        <button
          type="button"
          onClick={handleSubmit}
          className="px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700"
        >
          {isEdit ? "Save" : "Create"}
        </button>
      </div>
    </div>
  );
}
