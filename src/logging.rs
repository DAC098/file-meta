pub fn log_result<T, E>(value: Result<T, E>) -> Option<T>
where
    E: std::fmt::Display,
{
    match value {
        Ok(v) => Some(v),
        Err(err) => {
            println!("{}", err);
            None
        }
    }
}
