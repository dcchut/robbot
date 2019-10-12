use super::schema::countdowns;

#[derive(Queryable)]
pub struct Countdown {
    pub id: i32,
    pub end: i32,
    pub active: bool,
}

#[derive(Insertable)]
#[table_name = "countdowns"]
pub struct NewCountdown {
    pub end: i32,
    pub active: bool,
}
