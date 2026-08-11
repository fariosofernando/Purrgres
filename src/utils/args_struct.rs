use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "purrgres")]
#[command(about = "An automatic backup tool for PostgreSQL in Docker containers.")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    #[arg(short, long, help = "PostgreSQL User", required_unless_present_any = ["stats", "stop", "list_purrs", "server"])]
    pub user: Option<String>,

    #[arg(short, long, help = "Database name", required_unless_present_any = ["stats", "stop", "list_purrs", "server"])]
    pub database: Option<String>,

    #[arg(short, long, help = "Container name", required_unless_present_any = ["stats", "stop", "list_purrs", "server"])]
    pub container: Option<String>,

    #[arg(long, help = "Checks the status of the running backup.")]
    pub stats: bool,

    #[arg(long, help = "Stops the backup process")]
    pub stop: bool,

    #[arg(long, help = "List the backups performed")]
    pub list_purrs: bool,

    #[arg(long, help = "Applies a backup from a file")]
    pub rpurry: Option<String>,

    #[arg(long, help = "Start in server mode (receive backups via API)")]
    pub server: bool,

    #[arg(long, help = "Path to purrgres.toml config file")]
    pub config: Option<String>,

    #[arg(long, help = "Override server port", requires = "server")]
    pub port: Option<u16>,
}
