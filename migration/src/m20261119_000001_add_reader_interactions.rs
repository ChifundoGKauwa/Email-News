use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create().table(Readers::Table).if_not_exists()
                .col(ColumnDef::new(Readers::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Readers::Name).string().not_null())
                .col(ColumnDef::new(Readers::Email).string().not_null().unique_key())
                .col(ColumnDef::new(Readers::PasswordHash).string().not_null())
                .col(ColumnDef::new(Readers::CreatedAt).timestamp().not_null()).to_owned()
        ).await?;

        manager.create_table(
            Table::create().table(Claps::Table).if_not_exists()
                .col(ColumnDef::new(Claps::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Claps::ReaderId).uuid().not_null())
                .col(ColumnDef::new(Claps::StoryId).uuid().not_null())
                .col(ColumnDef::new(Claps::CreatedAt).timestamp().not_null())
                .foreign_key(ForeignKey::create().name("fk_claps_reader").from(Claps::Table, Claps::ReaderId).to(Readers::Table, Readers::Id).on_delete(ForeignKeyAction::Cascade))
                .foreign_key(ForeignKey::create().name("fk_claps_story").from(Claps::Table, Claps::StoryId).to(Newsletters::Table, Newsletters::Id).on_delete(ForeignKeyAction::Cascade)).to_owned()
        ).await?;
        manager.create_index(Index::create().name("uq_claps_reader_story").table(Claps::Table).col(Claps::ReaderId).col(Claps::StoryId).unique().to_owned()).await?;

        manager.create_table(
            Table::create().table(Comments::Table).if_not_exists()
                .col(ColumnDef::new(Comments::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Comments::ReaderId).uuid().not_null())
                .col(ColumnDef::new(Comments::StoryId).uuid().not_null())
                .col(ColumnDef::new(Comments::Body).text().not_null())
                .col(ColumnDef::new(Comments::CreatedAt).timestamp().not_null())
                .col(ColumnDef::new(Comments::UpdatedAt).timestamp().not_null())
                .foreign_key(ForeignKey::create().name("fk_comments_reader").from(Comments::Table, Comments::ReaderId).to(Readers::Table, Readers::Id).on_delete(ForeignKeyAction::Cascade))
                .foreign_key(ForeignKey::create().name("fk_comments_story").from(Comments::Table, Comments::StoryId).to(Newsletters::Table, Newsletters::Id).on_delete(ForeignKeyAction::Cascade)).to_owned()
        ).await?;

        manager.create_table(
            Table::create().table(Bookmarks::Table).if_not_exists()
                .col(ColumnDef::new(Bookmarks::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Bookmarks::ReaderId).uuid().not_null())
                .col(ColumnDef::new(Bookmarks::StoryId).uuid().not_null())
                .col(ColumnDef::new(Bookmarks::CreatedAt).timestamp().not_null())
                .foreign_key(ForeignKey::create().name("fk_bookmarks_reader").from(Bookmarks::Table, Bookmarks::ReaderId).to(Readers::Table, Readers::Id).on_delete(ForeignKeyAction::Cascade))
                .foreign_key(ForeignKey::create().name("fk_bookmarks_story").from(Bookmarks::Table, Bookmarks::StoryId).to(Newsletters::Table, Newsletters::Id).on_delete(ForeignKeyAction::Cascade)).to_owned()
        ).await?;
        manager.create_index(Index::create().name("uq_bookmarks_reader_story").table(Bookmarks::Table).col(Bookmarks::ReaderId).col(Bookmarks::StoryId).unique().to_owned()).await?;

        manager.create_table(
            Table::create().table(Highlights::Table).if_not_exists()
                .col(ColumnDef::new(Highlights::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Highlights::ReaderId).uuid().not_null())
                .col(ColumnDef::new(Highlights::StoryId).uuid().not_null())
                .col(ColumnDef::new(Highlights::Text).text().not_null())
                .col(ColumnDef::new(Highlights::StartOffset).integer().not_null())
                .col(ColumnDef::new(Highlights::EndOffset).integer().not_null())
                .col(ColumnDef::new(Highlights::CreatedAt).timestamp().not_null())
                .foreign_key(ForeignKey::create().name("fk_highlights_reader").from(Highlights::Table, Highlights::ReaderId).to(Readers::Table, Readers::Id).on_delete(ForeignKeyAction::Cascade))
                .foreign_key(ForeignKey::create().name("fk_highlights_story").from(Highlights::Table, Highlights::StoryId).to(Newsletters::Table, Newsletters::Id).on_delete(ForeignKeyAction::Cascade)).to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Highlights::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Bookmarks::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Comments::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Claps::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Readers::Table).if_exists().to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)] enum Readers { Table, Id, Name, Email, PasswordHash, CreatedAt }
#[derive(DeriveIden)] enum Claps { Table, Id, ReaderId, StoryId, CreatedAt }
#[derive(DeriveIden)] enum Comments { Table, Id, ReaderId, StoryId, Body, CreatedAt, UpdatedAt }
#[derive(DeriveIden)] enum Bookmarks { Table, Id, ReaderId, StoryId, CreatedAt }
#[derive(DeriveIden)] enum Highlights { Table, Id, ReaderId, StoryId, Text, StartOffset, EndOffset, CreatedAt }
#[derive(DeriveIden)] enum Newsletters { Table, Id }
