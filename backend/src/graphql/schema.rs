use async_graphql::{MergedObject, Object};

use crate::graphql::accounts::{AccountMutation, AccountQuery};
use crate::graphql::bank_entries::{BankEntryMutation, BankEntryQuery};
use crate::graphql::bank_imports::{BankImportMutation, BankImportQuery};
use crate::graphql::projected_entries::{ProjectedEntryMutation, ProjectedEntryQuery};

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    async fn health(&self) -> &str {
        "ok"
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(HealthQuery, AccountQuery, BankEntryQuery, BankImportQuery, ProjectedEntryQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(AccountMutation, BankEntryMutation, BankImportMutation, ProjectedEntryMutation);
