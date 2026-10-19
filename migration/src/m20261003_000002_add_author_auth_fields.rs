use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("authors"))
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("password_hash"))
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("password_salt"))
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("email_hash"))
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("email_salt"))
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
                    .table(Alias::new("authors"))
                    .drop_column(Alias::new("email_salt"))
                    .drop_column(Alias::new("email_hash"))
                    .drop_column(Alias::new("password_salt"))
                    .drop_column(Alias::new("password_hash"))
                    .to_owned(),
            )
            .await
    }
}
