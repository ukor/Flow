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
                    .col(blob(PassKey::CredentialId).unique_key().not_null())
                    .col(blob(PassKey::PublicKey).not_null())
                    .col(integer(PassKey::SignCount).default(0).not_null())
                    .col(string(PassKey::Attestation))
                    .col(text(PassKey::JsonData).not_null())
                    .col(timestamp_with_time_zone(PassKey::TimeCreated).not_null())
                    .to_owned(),
            )
            .await?;

        // Create index on device_id for fast lookups during registration
        // (to query existing credentials for exclude_credentials)
        manager
            .create_index(
                Index::create()
                    .name("idx_passkey_device_id")
                    .table(PassKey::Table)
                    .col(PassKey::DeviceId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indices first (if database requires it)
        manager
            .drop_index(
                Index::drop()
                    .name("idx_passkey_device_id")
                    .table(PassKey::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(PassKey::Table).to_owned())
            .await?;

        Ok(())
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
