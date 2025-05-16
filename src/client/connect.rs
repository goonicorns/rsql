// rsql.rs - Nathanael "NateNateNate" Thevarajah
// <natenatenat3@protonmail.com> - Refer to the license for more
// information.

use crate::engine::connection::DbConfig;

use crate::client::app;

use clap::{Parser, ValueEnum};
use mysql::PooledConn;

/// rsql.rs - TUI SQL Client.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // The sql variant to use.
    pub sqlt: Variant,

    #[command(flatten)]
    // The connection details.
    pub connection: CommonConnection,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CommonConnection {
    #[arg(long)]
    // The username.
    pub username: String,

    #[arg(long)]
    // The db.
    pub db: String,

    #[arg(long)]
    // The password.
    pub password: String,

    #[arg(long)]
    // The host, i.e. 127.0.0.1.
    pub host: Option<String>,

    #[arg(long)]
    // The port, i.e. 3306.
    pub port: Option<i32>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Variant {
    Mysql,
}

impl Args {
    pub fn connect(&self) -> color_eyre::Result<()> {
        let host = self
            .connection
            .host
            .clone()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = self.connection.port.clone().unwrap_or_else(|| 3306);

        let db = DbConfig {
            name: self.connection.db.clone(),
            host,
            user: self.connection.username.clone(),
            password: self.connection.password.clone(),
            port,
        };
        let conn = db.connect();
        // @TODO perhaps look into the error messages retured by
        // MySqlError
        // clearer. https://docs.rs/mysql/latest/mysql/struct.MySqlError.html
        match conn {
            Ok(val) => launch_tui(val),
            Err(e) => Ok(match e {
                _ => println!("Uncaught error: {:?}", e.to_string()),
            }),
        }
    }
}

fn launch_tui(_connection: PooledConn) -> color_eyre::Result<()> {
    app::run()
}
