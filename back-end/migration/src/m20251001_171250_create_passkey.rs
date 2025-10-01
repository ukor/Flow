use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PassKey::Table)
                    .if_not_exists()
                    .col(pk_auto(PassKey::Id))
                    .col(string(PassKey::DeviceId))
                    .col(blob(PassKey::CredentialId))
                    .col(blob(PassKey::PublicKey))
                    .col(integer(PassKey::SignCount))
                    .col(string(PassKey::Attestation))
                    .col(text(PassKey::JsonData))
                    .col(timestamp_with_time_zone(PassKey::TimeCreated))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PassKey::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PassKey {
    Table,
    Id,
    DeviceId,
    CredentialId,
    PublicKey,
    SignCount,
    Attestation,
    JsonData,
    TimeCreated,
}
