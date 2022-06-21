use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_humanize::HumanTime;
use sqlx::{Pool, Sqlite};
use std::ops::Sub;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Countdown {
    id: i64,
    end: i64,
    active: bool,
    guild: i64,
}

impl Countdown {
    pub fn as_pretty_string(&self, current_dt: &DateTime<Utc>) -> String {
        // Drop the sub-second component of the duration here so we don't end up with an
        // absurdly accurate text representation of this duration.
        let duration =
            Duration::seconds((-current_dt.sub(Utc.timestamp(self.end as i64, 0))).num_seconds());

        format!("***S{}*** is {:#}.", self.id, HumanTime::from(duration),)
    }
}

pub struct CountdownStore<'pool> {
    pool: &'pool Pool<Sqlite>,
}

impl<'pool> CountdownStore<'pool> {
    pub fn new(pool: &'pool Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Inserts a new countdown with the given end timestamp into the database.
    pub async fn insert(&self, timestamp: i64, guild_id: i64) -> Result<()> {
        sqlx::query!(
            "
        INSERT INTO countdowns (end, active, guild)
        VALUES (?, true, ?)
            ",
            timestamp,
            guild_id
        )
        .execute(self.pool)
        .await
        .map(|_| ())
        .map_err(|_| anyhow!("failed to insert countdown ending at {}", timestamp))
    }

    /// Returns the first `limit` timestamps for the given guild at or after the input `timestamp`.
    /// Countdowns are returned from oldest end to newest end.
    pub async fn get_after(
        &self,
        timestamp: i64,
        guild_id: i64,
        limit: i64,
    ) -> Result<Vec<Countdown>> {
        sqlx::query_as!(
            Countdown,
            r#"
        SELECT id as "id!", end as "end!", active as "active!", guild as "guild!"
        FROM countdowns
        WHERE end >= ?
        AND guild = ?
        AND active = true
        ORDER BY end ASC, id ASC
        LIMIT ?
            "#,
            timestamp,
            guild_id,
            limit,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|_| anyhow!("failed to get countdowns after {}", timestamp))
    }

    pub async fn get_first_after(
        &self,
        timestamp: i64,
        guild_id: i64,
    ) -> Result<Option<Countdown>> {
        self.get_after(timestamp, guild_id, 1)
            .await
            .map(|mut cs| cs.pop())
    }
}
