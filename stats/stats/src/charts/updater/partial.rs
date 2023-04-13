use super::get_min_block_blockscout;
use crate::{
    charts::{
        find_chart,
        insert::{insert_data_many, DateValue},
    },
    metrics, Chart, UpdateError,
};
use async_trait::async_trait;
use chrono::NaiveDate;
use entity::chart_data;
use sea_orm::{prelude::*, FromQueryResult, QueryOrder, QuerySelect};

#[async_trait]
pub trait ChartPartialUpdater: Chart {
    async fn get_values(
        &self,
        blockscout: &DatabaseConnection,
        last_row: Option<DateValue>,
    ) -> Result<Vec<DateValue>, UpdateError>;

    async fn update_with_values(
        &self,
        db: &DatabaseConnection,
        blockscout: &DatabaseConnection,
        force_full: bool,
    ) -> Result<(), UpdateError> {
        let chart_id = find_chart(db, self.name())
            .await
            .map_err(UpdateError::StatsDB)?
            .ok_or_else(|| UpdateError::NotFound(self.name().into()))?;
        let min_blockscout_block = get_min_block_blockscout(blockscout)
            .await
            .map_err(UpdateError::BlockscoutDB)?;
        let last_row = get_last_row(self, chart_id, min_blockscout_block, db, force_full).await?;
        let values = {
            let _timer = metrics::CHART_FETCH_NEW_DATA_TIME
                .with_label_values(&[self.name()])
                .start_timer();
            self.get_values(blockscout, last_row)
                .await?
                .into_iter()
                .map(|value| value.active_model(chart_id, Some(min_blockscout_block)))
        };
        insert_data_many(db, values)
            .await
            .map_err(UpdateError::StatsDB)?;
        Ok(())
    }
}

#[derive(Debug, FromQueryResult)]
struct SyncInfo {
    pub date: NaiveDate,
    pub value: String,
    pub min_blockscout_block: Option<i64>,
}

async fn get_last_row<C>(
    chart: &C,
    chart_id: i32,
    min_blockscout_block: i64,
    db: &DatabaseConnection,
    force_full: bool,
) -> Result<Option<DateValue>, UpdateError>
where
    C: Chart + ?Sized,
{
    let last_row = if force_full {
        tracing::info!(
            min_blockscout_block = min_blockscout_block,
            chart = chart.name(),
            "running full update due to force override"
        );
        None
    } else {
        let last_row: Option<SyncInfo> = chart_data::Entity::find()
            .column(chart_data::Column::Date)
            .column(chart_data::Column::Value)
            .column(chart_data::Column::MinBlockscoutBlock)
            .filter(chart_data::Column::ChartId.eq(chart_id))
            .order_by_desc(chart_data::Column::Date)
            .offset(1)
            .into_model()
            .one(db)
            .await
            .map_err(UpdateError::StatsDB)?;

        match last_row {
            Some(row) => {
                if let Some(block) = row.min_blockscout_block {
                    if block == min_blockscout_block {
                        tracing::info!(
                            min_blockscout_block = min_blockscout_block,
                            min_chart_block = block,
                            chart = chart.name(),
                            "running partial update"
                        );
                        Some(DateValue {
                            date: row.date,
                            value: row.value,
                        })
                    } else {
                        tracing::info!(
                            min_blockscout_block = min_blockscout_block,
                            min_chart_block = block,
                            chart = chart.name(),
                            "running full update due to min blocks mismatch"
                        );
                        None
                    }
                } else {
                    tracing::info!(
                        min_blockscout_block = min_blockscout_block,
                        chart = chart.name(),
                        "running full update due to lack of saved min block"
                    );
                    None
                }
            }
            None => {
                tracing::info!(
                    min_blockscout_block = min_blockscout_block,
                    chart = chart.name(),
                    "running full update due to lack of history data"
                );
                None
            }
        }
    };

    Ok(last_row)
}
