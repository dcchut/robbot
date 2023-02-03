use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

struct RockCount {
    user_id: i64,
    count: i64,
}

pub struct RockCounter<'pool> {
    pool: &'pool Pool<Sqlite>,
}

impl<'pool> RockCounter<'pool> {
    pub fn new(pool: &'pool Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Increments the rock count for the given user ID
    pub async fn increment(&self, user_id: i64) -> Result<i64> {
        // Make sure the user ID isn't too big for our DB
        // let user_id =
        //     i64::try_from(user_id).with_context(|| format!("user id {} too large", user_id))?;

        // FIXME: surely there's a way to do this with a single statement?
        //        my SQL-foo is weak.
        let mut tx = self.pool.begin().await?;

        let mut rock_count = sqlx::query_as!(
            RockCount,
            "SELECT user_id, count FROM rocks WHERE user_id = ?",
            user_id
        )
        .fetch_optional(&mut tx)
        .await
        .with_context(|| format!("failed to get count for user {user_id}"))?
        .unwrap_or(RockCount { user_id, count: 0 });

        rock_count.count += 1;

        sqlx::query!(
            "INSERT OR REPLACE INTO rocks (user_id, count) VALUES (?, ?)",
            rock_count.user_id,
            rock_count.count
        )
        .execute(&mut tx)
        .await
        .with_context(|| format!("failed to update rock count for user {user_id}"))?;

        tx.commit().await?;

        Ok(rock_count.count)
    }
}
