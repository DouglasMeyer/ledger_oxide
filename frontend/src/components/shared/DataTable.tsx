import { ReactNode } from "react";

interface Column<T> {
  key: string;
  header: string;
  render: (row: T) => ReactNode;
  className?: string;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  rows: T[];
  keyExtractor: (row: T) => string | number;
}

export default function DataTable<T>({ columns, rows, keyExtractor }: DataTableProps<T>) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse">
        <thead>
          <tr className="border-b bg-gray-50">
            {columns.map((col) => (
              <th key={col.key} className={`text-left px-3 py-2 text-sm font-medium text-gray-600 ${col.className || ""}`}>
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={keyExtractor(row)} className="border-b hover:bg-gray-50">
              {columns.map((col) => (
                <td key={col.key} className={`px-3 py-2 text-sm ${col.className || ""}`}>
                  {col.render(row)}
                </td>
              ))}
            </tr>
          ))}
          {rows.length === 0 && (
            <tr>
              <td colSpan={columns.length} className="px-3 py-8 text-center text-gray-400">
                No data
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
