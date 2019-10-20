use std::sync::Arc;

use crate::schema::countdowns;
use chrono::{DateTime, TimeZone, Utc};
use chrono_humanize::HumanTime;
use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};
use serenity::utils::Mutex;
use std::ops::Sub;

#[derive(Queryable, Debug, Clone)]
pub struct Countdown {
    pub id: i32,
    pub end: i32,
    pub active: bool,
}

impl Countdown {
    pub fn as_pretty_string(&self, current_dt: &DateTime<Utc>) -> String {
        format!(
            "***S{}*** is {:#}.",
            self.id,
            HumanTime::from(-current_dt.sub(Utc.timestamp(self.end as i64, 0)))
        )
    }
}

#[derive(Insertable)]
#[table_name = "countdowns"]
struct NewCountdown {
    pub end: i32,
    pub active: bool,
}

/// Inserts a new countdown with the given `timestamp` into the db.
pub async fn insert_countdown(timestamp: i32, conn: &Arc<Mutex<SqliteConnection>>) -> bool {
    let new_countdown = NewCountdown {
        end: timestamp,
        active: true,
    };

    let conn = conn.lock().await;

    // In practice this _should_ never error
    diesel::insert_into(crate::schema::countdowns::table)
        .values(&new_countdown)
        .execute(&*conn)
        .is_ok()
}

/// Returns (if possible) the first countdown that would trigger after `dt`.
pub async fn get_first_countdown(
    dt: &DateTime<Utc>,
    conn: &Arc<Mutex<SqliteConnection>>,
) -> Option<Countdown> {
    get_countdowns(1, dt, conn).await.pop()
}

/// Returns a list of the at most `limit` countdowns that would trigger after `dt`.
pub async fn get_countdowns(
    limit: i64,
    dt: &DateTime<Utc>,
    conn: &Arc<Mutex<SqliteConnection>>,
) -> Vec<Countdown> {
    use crate::schema::countdowns::dsl::*;

    // TODO: figure out a solution for SQLITE only taking i32's here
    let timestamp = dt.timestamp() as i32;

    countdowns
        .filter(active.eq(true))
        .filter(end.ge(timestamp))
        .order(end.asc())
        .limit(limit)
        .load::<Countdown>(&*conn.lock().await)
        .unwrap_or_else(|_| {
            panic!(
                "get_countdowns with limit={}, dt={} failed to query DB",
                limit, dt
            )
        })
}
