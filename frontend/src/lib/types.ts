export interface Account {
  id: number;
  name: string;
  balanceCents: number;
  active: boolean;
  asset: boolean | null;
  category: string | null;
  position: number | null;
}

export interface AccountEntry {
  id: number;
  accountId: number;
  bankEntryId: number;
  amountCents: number;
  notes: string | null;
  account: Account | null;
}

export interface BankEntry {
  id: number;
  date: string;
  amountCents: number;
  description: string | null;
  notes: string | null;
  externalId: string | null;
  createdAt: string;
  updatedAt: string;
  accountEntries: AccountEntry[];
}
