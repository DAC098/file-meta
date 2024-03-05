pub type DateTime = chrono::DateTime<chrono::Utc>;

pub fn datetime_now() -> DateTime {
    chrono::Utc::now()
}
