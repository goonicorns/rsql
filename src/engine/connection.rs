// rsql.rs - Nathanael "NateNateNate" Thevarajah
// <natenatenat3@protonmail.com> - Refer to the license for more
// information.

use mysql::*;

/// The database configuration used for SQL connections.
///
/// # Example
///
/// ```
/// let db = DbConfig {
///     name: "rsql".to_string(),
///     host: "127.0.0.1".to_string(),
///     user: "nate".to_string(),
///     password: "123".to_string(),
///     port: 3306,
/// };
/// ```
pub struct DbConfig {
    pub name: String,
    pub host: String,
    pub user: String,
    pub password: String,
    pub port: i32,
}

/// TODO Create a trait as we plan to support other SQL variants
/// besides Mysql.
impl DbConfig {
    /// Returns the mysql connection. I'm not good at writing
    /// documentation or comments, so that's about it :P
    pub fn connect(&self) -> mysql::Result<PooledConn> {
        let url = format!(
            "mysql://{user}:{password}@{host}:{port}/{db_name}",
            user = self.user,
            password = self.password,
            host = self.host,
            port = self.port,
            db_name = self.name
        );
        Opts::try_from(url.as_ref())?;
        let pool = Pool::new(url.as_ref())?;
        return Ok(pool.get_conn()?);
    }
}

/// TODO better test, we cannot be testing it like this xd If we
/// continue like this, we'll have to hardcode the connection details
/// into the test, which breaks shii. For now, this is fine, but it's
/// not great. What i want is to test that we get the proper return
/// type, which would include failures to connect without the test
/// fucking failing.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conn_can() {
        let db = DbConfig {
            name: "rsql".to_string(),
            host: "127.0.0.1".to_string(),
            user: "nate".to_string(),
            password: "123".to_string(),
            port: 3306,
        };

        let result = db.connect();
        assert!(result.is_ok(), "DB connection failed: {:?}", result);
    }
}
