mod shared;
mod host;
mod cli;

use clap::Parser;

use cli::commands;
use cli::output::OutputFormat;

#[derive(Parser)]
#[command(name = "ptd", about = "CLI for PT-Depiler browser extension")]
struct Cli {
    /// Select browser/profile instance (prefix match supported)
    #[arg(long, global = true, env = "PTD_INSTANCE")]
    instance: Option<String>,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value = "30")]
    timeout: u64,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "json")]
    format: OutputFormat,

    /// Alias for --format pretty
    #[arg(long, global = true)]
    pretty: bool,

    /// Alias for --format table
    #[arg(long, global = true)]
    table: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Register the native messaging host manifest
    Install(commands::install::InstallArgs),
    /// Remove the native messaging host manifest
    Uninstall(commands::uninstall::UninstallArgs),
    /// Show running PT-Depiler instances
    Status,
    /// Search torrents on a site
    Search(commands::search::SearchArgs),
    /// Download a torrent
    Download(commands::download::DownloadArgs),
    /// Query downloader status/config/version
    Downloader(commands::downloader::DownloaderArgs),
    /// Manage download history
    DownloadHistory(commands::download_history::DownloadHistoryArgs),
    /// Query site config or favicon
    Site(commands::site::SiteArgs),
    /// Query or manage user info
    UserInfo(commands::user_info::UserInfoArgs),
    /// Manage keep-upload (cross-seeding) tasks
    KeepUpload(commands::keep_upload::KeepUploadArgs),
}

fn main() {
    let cli = Cli::parse();

    let format = if cli.pretty {
        OutputFormat::Pretty
    } else if cli.table {
        OutputFormat::Table
    } else {
        cli.format
    };

    let result = match cli.command {
        Commands::Install(args) => commands::install::run(args),
        Commands::Uninstall(args) => commands::uninstall::run(args),
        Commands::Status => commands::status::run(),
        Commands::Search(args) => commands::search::run(args, cli.instance.as_deref(), cli.timeout, format),
        Commands::Download(args) => commands::download::run(args, cli.instance.as_deref(), cli.timeout, format),
        Commands::Downloader(args) => commands::downloader::run(args, cli.instance.as_deref(), cli.timeout, format),
        Commands::DownloadHistory(args) => commands::download_history::run(args, cli.instance.as_deref(), cli.timeout, format),
        Commands::Site(args) => commands::site::run(args, cli.instance.as_deref(), cli.timeout, format),
        Commands::UserInfo(args) => commands::user_info::run(args, cli.instance.as_deref(), cli.timeout, format),
        Commands::KeepUpload(args) => commands::keep_upload::run(args, cli.instance.as_deref(), cli.timeout, format),
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
