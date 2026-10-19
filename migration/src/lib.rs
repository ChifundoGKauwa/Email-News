pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20220101_000002_create_authors_and_newsletters;
mod m20261003_000001_add_subscription_confirmation_fields;
mod m20261003_000002_add_author_auth_fields;
mod m20261007_000003_add_image_url_to_newsletters;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20220101_000002_create_authors_and_newsletters::Migration),
            Box::new(m20261003_000001_add_subscription_confirmation_fields::Migration),
            Box::new(m20261003_000002_add_author_auth_fields::Migration),
            Box::new(m20261007_000003_add_image_url_to_newsletters::Migration),
        ]
    }
}
