use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Authors::Table)
                    .if_not_exists()
                    .col(pk_uuid(Authors::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(string(Authors::Email).unique_key().not_null())
                    .col(string(Authors::Name).not_null())
                    .col(string_null(Authors::Bio))
                    .col(date_time(Authors::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        let mut fk = ForeignKey::create();
        fk.name("fk_newsletters_author")
            .from(Newsletters::Table, Newsletters::AuthorId)
            .to(Authors::Table, Authors::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade);

        manager
            .create_table(
                Table::create()
                    .table(Newsletters::Table)
                    .if_not_exists()
                    .col(pk_uuid(Newsletters::Id).default(Expr::cust("gen_random_uuid()")))
                    .col(uuid(Newsletters::AuthorId).not_null())
                    .col(string(Newsletters::Title).not_null())
                    .col(text(Newsletters::Content).not_null())
                    .col(boolean(Newsletters::Published).not_null().default(false))
                    .col(date_time(Newsletters::CreatedAt).not_null())
                    .col(date_time(Newsletters::UpdatedAt).not_null())
                    .foreign_key(&mut fk)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Newsletters::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Authors::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Authors {
    Table,
    Id,
    Email,
    Name,
    Bio,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Newsletters {
    Table,
    Id,
    AuthorId,
    Title,
    Content,
    Published,
    CreatedAt,
    UpdatedAt,
}
