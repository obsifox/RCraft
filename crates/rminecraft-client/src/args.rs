use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "rminecraft-client")]
#[command(author = "rMinecraft Team")]
#[command(version = "1.12.2-rust")]
#[command(disable_version_flag = true)]
#[command(about = "Native Rust Minecraft 1.12.2 Client")]
pub struct ClientArgs {
    #[arg(long, default_value = "RustPlayer")]
    pub username: String,

    #[arg(long)]
    pub uuid: Option<String>,

    #[arg(long, default_value = "1.12.2-rust")]
    pub version: String,

    #[arg(long = "gameDir", alias = "game-dir", default_value = ".minecraft")]
    pub game_dir: String,

    #[arg(long = "assetsDir", alias = "assets-dir", default_value = ".minecraft/assets")]
    pub assets_dir: String,

    #[arg(long = "assetIndex", alias = "asset-index", default_value = "1.12")]
    pub asset_index: String,

    #[arg(long, default_value = "127.0.0.1:25565")]
    pub server: String,

    #[arg(long)]
    pub skin: Option<String>,

    #[arg(long)]
    pub cape: Option<String>,
}
