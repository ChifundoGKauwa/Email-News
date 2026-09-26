use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscribers::Table)
                    .if_not_exists()
                    .col(pk_uuid(Subscribers::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string(Subscribers::Email).unique_key().not_null())
                    .col(string(Subscribers::Name))
                    .col(date_time(Subscribers::SubscribedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscribers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Subscribers {
    Table,
    Id,
    Email,
    Name,
    SubscribedAt,
}
