use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

use super::feedback::ValidatedFeedback;

#[derive(Clone)]
#[cfg_attr(test, derive(Default))]
pub struct FeedbackStore {
    pool: Option<PgPool>,
    #[cfg(test)]
    entries: std::sync::Arc<std::sync::RwLock<Vec<FeedbackRecord>>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FeedbackRecord {
    pub id: Uuid,
    pub category: String,
    pub message: String,
    pub created_at: String,
}

impl FeedbackStore {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .after_connect(|connection, _| {
                Box::pin(async move {
                    sqlx::query("SET statement_timeout = '5s'")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("SET lock_timeout = '2s'")
                        .execute(&mut *connection)
                        .await?;
                    Ok(())
                })
            })
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(url)
            .await?;
        // Fail startup when the schema has not been installed. Runtime cannot create tables.
        sqlx::query("SELECT id, category, message, created_at FROM feedback LIMIT 0")
            .execute(&pool)
            .await?;
        Ok(Self {
            pool: Some(pool),
            #[cfg(test)]
            entries: Default::default(),
        })
    }

    pub async fn submit(&self, feedback: ValidatedFeedback) -> Result<FeedbackRecord, sqlx::Error> {
        let id = Uuid::new_v4();
        let category = feedback.category.as_str().to_owned();
        if let Some(pool) = &self.pool {
            let row = sqlx::query("INSERT INTO feedback (id, category, message) VALUES ($1, $2, $3) RETURNING to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS created_at")
                .bind(id).bind(&category).bind(&feedback.message).fetch_one(pool).await?;
            return Ok(FeedbackRecord {
                id,
                category,
                message: feedback.message,
                created_at: row.try_get("created_at")?,
            });
        }
        #[cfg(test)]
        {
            let record = FeedbackRecord {
                id,
                category,
                message: feedback.message,
                created_at: "2026-01-01T00:00:00Z".into(),
            };
            self.entries.write().unwrap().push(record.clone());
            Ok(record)
        }
        #[cfg(not(test))]
        Err(sqlx::Error::PoolClosed)
    }

    pub async fn list_page(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<FeedbackRecord>, sqlx::Error> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SELECT id, category, message, to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS created_at FROM feedback ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2")
                .bind(limit as i64).bind(offset as i64).fetch_all(pool).await?;
            return rows
                .into_iter()
                .map(|row| {
                    Ok(FeedbackRecord {
                        id: row.try_get("id")?,
                        category: row.try_get("category")?,
                        message: row.try_get("message")?,
                        created_at: row.try_get("created_at")?,
                    })
                })
                .collect();
        }
        #[cfg(test)]
        {
            Ok(self
                .entries
                .read()
                .unwrap()
                .iter()
                .rev()
                .skip(offset)
                .take(limit)
                .cloned()
                .collect())
        }
        #[cfg(not(test))]
        Err(sqlx::Error::PoolClosed)
    }
}
