import { useCallback } from "react";
import CurrencyInput from "./CurrencyInput";
import AccountSelect from "./AccountSelect";

interface SplitRow {
  key: string;
  accountName: string;
  amountCents: number;
}

interface SplitRowsWidgetProps {
  accounts: { id: number; name: string }[];
  rows: SplitRow[];
  totalCents: number;
  onChange: (rows: SplitRow[]) => void;
}

export default function SplitRowsWidget({ accounts, rows, totalCents, onChange }: SplitRowsWidgetProps) {
  const allocated = rows.reduce((s, r) => s + r.amountCents, 0);
  const remaining = totalCents - allocated;

  const addRow = useCallback(() => {
    const key = `${Date.now()}-${Math.random()}`;
    onChange([...rows, { key, accountName: "", amountCents: 0 }]);
  }, [rows, onChange]);

  const removeRow = useCallback(
    (key: string) => {
      onChange(rows.filter((r) => r.key !== key));
    },
    [rows, onChange],
  );

  const updateRow = useCallback(
    (key: string, field: keyof SplitRow, val: string | number) => {
      onChange(
        rows.map((r) => (r.key === key ? { ...r, [field]: val } : r)),
      );
    },
    [rows, onChange],
  );

  return (
    <div className="space-y-1">
      {rows.map((row) => (
        <div key={row.key} className="flex gap-2 items-center">
          <AccountSelect
            accounts={accounts}
            value={row.accountName}
            onChange={(name) => updateRow(row.key, "accountName", name)}
            className="flex-1"
          />
          <CurrencyInput
            value={row.amountCents}
            onChange={(cents) => updateRow(row.key, "amountCents", cents)}
          />
          <button
            type="button"
            onClick={() => removeRow(row.key)}
            className="text-red-500 hover:text-red-700 px-1"
          >
            ×
          </button>
        </div>
      ))}
      <div className="flex justify-between text-sm">
        <button type="button" onClick={addRow} className="text-blue-600 hover:underline">
          + Add split
        </button>
        <span className={remaining === 0 ? "text-green-600" : "text-red-600"}>
          {remaining >= 0
            ? `${(remaining / 100).toFixed(2)} remaining`
            : `${Math.abs(remaining / 100).toFixed(2)} over`}
        </span>
      </div>
    </div>
  );
}
