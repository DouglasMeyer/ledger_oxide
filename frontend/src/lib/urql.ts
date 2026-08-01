import { Client, cacheExchange, fetchExchange, gql } from "urql";

export const client = new Client({
  url: "/graphql",
  exchanges: [cacheExchange, fetchExchange],
});

export const ACCOUNTS_QUERY = gql`
  query Accounts($active: Boolean) {
    accounts(active: $active) {
      id
      name
      balanceCents
      active
      asset
      category
      position
    }
  }
`;

export const ACCOUNT_QUERY = gql`
  query Account($id: ID!) {
    account(id: $id) {
      id
      name
      balanceCents
      active
      asset
      category
      position
    }
  }
`;

export const BANK_ENTRIES_QUERY = gql`
  query BankEntries($dateFrom: NaiveDate, $dateTo: NaiveDate, $accountId: Int) {
    bankEntries(dateFrom: $dateFrom, dateTo: $dateTo, accountId: $accountId) {
      id
      date
      amountCents
      description
      notes
      externalId
      createdAt
      updatedAt
      accountEntries {
        id
        accountId
        bankEntryId
        amountCents
        notes
        account {
          id
          name
          balanceCents
          active
          asset
          category
          position
        }
      }
    }
  }
`;

export const BANK_ENTRY_QUERY = gql`
  query BankEntry($id: ID!) {
    bankEntry(id: $id) {
      id
      date
      amountCents
      description
      notes
      externalId
      createdAt
      updatedAt
      accountEntries {
        id
        accountId
        bankEntryId
        amountCents
        notes
        account {
          id
          name
          balanceCents
          active
          asset
          category
          position
        }
      }
    }
  }
`;

export const CREATE_BANK_ENTRY = gql`
  mutation CreateBankEntry($input: CreateBankEntryInput!) {
    createBankEntry(input: $input) {
      id
      date
      amountCents
      description
      notes
      accountEntries {
        id
        amountCents
        account {
          id
          name
        }
      }
    }
  }
`;

export const UPDATE_BANK_ENTRY = gql`
  mutation UpdateBankEntry($id: ID!, $input: UpdateBankEntryInput!) {
    updateBankEntry(id: $id, input: $input) {
      id
      date
      amountCents
      description
      notes
      accountEntries {
        id
        amountCents
        account {
          id
          name
        }
      }
    }
  }
`;

export const DELETE_BANK_ENTRY = gql`
  mutation DeleteBankEntry($id: ID!) {
    deleteBankEntry(id: $id) {
      id
    }
  }
`;

export const BANK_IMPORTS_QUERY = gql`
  query BankImports {
    bankImports {
      id
      balanceCents
      createdAt
    }
  }
`;

export const UNALLOCATED_QUERY = gql`
  query BankEntriesNeedingDistribution {
    bankEntriesNeedingDistribution {
      id
      date
      amountCents
      description
      notes
      externalId
      accountEntries {
        id
        amountCents
        account {
          id
          name
        }
      }
    }
  }
`;
