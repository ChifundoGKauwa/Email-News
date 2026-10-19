use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("subscribers"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("status"))
                            .string()
                            .not_null()
                            .default("pending_confirmation"),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("confirmation_token"))
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("subscribers"))
                    .drop_column(Alias::new("confirmation_token"))
                    .drop_column(Alias::new("status"))
                    .to_owned(),
            )
            .await
    }
}
