import { useState } from "react";
import { useMutation, gql } from "urql";

const IMPORT_MUTATION = gql`
  mutation ImportBankStatement($input: ImportBankStatementInput!) {
    importBankStatement(input: $input) {
      bankImport {
        id
        balanceCents
        createdAt
      }
      createdCount
      skippedCount
      entries {
        id
        externalId
        date
        amountCents
        description
        wasSkipped
      }
    }
  }
`;

export default function ImportPage() {
  const [fileContent, setFileContent] = useState("");
  const [importResult, executeImport] = useMutation(IMPORT_MUTATION);

  const handleFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const text = await file.text();
    setFileContent(text);
  };

  const handleImport = () => {
    if (!fileContent.trim()) return;
    executeImport({ input: { fileContent } });
  };

  return (
    <div>
      <h1 className="text-2xl font-bold mb-4">Import Statement</h1>
      <p className="text-gray-500 mb-4">Upload an OFX or QFX file to import bank transactions.</p>

      <input type="file" accept=".ofx,.qfx" onChange={handleFile} className="mb-3 block" />

      <button
        onClick={handleImport}
        disabled={!fileContent || importResult.fetching}
        className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
      >
        {importResult.fetching ? "Importing…" : "Import"}
      </button>

      {importResult.error && (
        <div className="mt-4 p-3 bg-red-50 text-red-700 rounded">
          {importResult.error.message}
        </div>
      )}

      {importResult.data && (
        <div className="mt-4 p-4 bg-green-50 rounded">
          <p className="font-semibold">
            Imported {importResult.data.importBankStatement.createdCount} transaction
            {importResult.data.importBankStatement.createdCount !== 1 ? "s" : ""}
            {importResult.data.importBankStatement.skippedCount > 0 &&
              ` (${importResult.data.importBankStatement.skippedCount} skipped as duplicates)`}
          </p>
          <p className="text-sm text-gray-500">
            Statement balance: ${(importResult.data.importBankStatement.bankImport.balanceCents / 100).toFixed(2)}
          </p>
        </div>
      )}
    </div>
  );
}
