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
///     user: "donald".to_string(),
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
    /// Returns the mysql connection.
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
