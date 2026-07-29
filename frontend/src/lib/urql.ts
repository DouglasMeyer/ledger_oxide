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

export const BANK_ENTRIES_QUERY = gql`
  query BankEntries($dateFrom: Date, $dateTo: Date, $accountId: Int) {
    bankEntries(dateFrom: $dateFrom, dateTo: $dateTo, accountId: $accountId) {
      id
      date
      amountCents
      description
      notes
      externalId
      createdAt
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
