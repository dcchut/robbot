use std::sync::Arc;

use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};

use crate::schema::rocks;

#[derive(Queryable, Debug, Clone)]
pub struct Rocks {
    pub user_id: i32,
    pub count: i32,
}

#[derive(Insertable)]
#[table_name = "rocks"]
struct RocksUpdate {
    pub user_id: i32,
}

pub async fn update_rocks(user_id: u64, conn: &Arc<std::sync::Mutex<SqliteConnection>>) -> i32 {
    let user_id = user_id as i32;
    let conn = conn.lock().expect("Unable to acquire mutex");

    let current_count = rocks::table.find(user_id).first::<Rocks>(&*conn);

    match current_count {
        Ok(rock) => {
            diesel::update(rocks::table.find(user_id))
                .set(rocks::count.eq(rock.count + 1))
                .execute(&*conn)
                .expect("error updating count");
        }
        Err(_) => {
            diesel::insert_into(rocks::table)
                .values(&RocksUpdate { user_id })
                .execute(&*conn)
                .expect("error increasing count");
        }
    };

    let current_count = rocks::table
        .find(user_id)
        .first::<Rocks>(&*conn)
        .expect("failed to acquire count");

    current_count.count
}
