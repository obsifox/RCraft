use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use rminecraft_protocol::offline_player_uuid;

#[derive(Parser, Debug)]
#[command(name = "rminecraft-launcher")]
#[command(about = "Official Launcher for rMinecraft 1.12.2 Rust Edition")]
struct Cli {
    #[arg(long, default_value = ".minecraft")]
    work_dir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the standard .minecraft directory structure
    Init,
    /// Add or update a local offline account
    AddAccount {
        #[arg(short, long)]
        username: String,
        #[arg(long)]
        skin: Option<String>,
        #[arg(long)]
        cape: Option<String>,
    },
    /// List all local accounts
    ListAccounts,
    /// Launch rMinecraft 1.12.2 client
    Launch {
        #[arg(short, long)]
        username: String,
        #[arg(short, long, default_value = "127.0.0.1:25565")]
        server: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LauncherProfiles {
    profiles: HashMap<String, Profile>,
    #[serde(rename = "selectedUser")]
    selected_user: Option<SelectedUser>,
    #[serde(rename = "authenticationDatabase")]
    auth_database: HashMap<String, AuthUser>,
    #[serde(rename = "clientToken")]
    client_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Profile {
    name: String,
    #[serde(rename = "lastVersionId")]
    last_version_id: String,
    #[serde(rename = "created")]
    created: String,
    #[serde(rename = "type")]
    profile_type: String,
    skin_path: Option<String>,
    cape_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SelectedUser {
    account: String,
    profile: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AuthUser {
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "username")]
    username: String,
    uuid: String,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let base_path = PathBuf::from(&cli.work_dir);

    match cli.command {
        Commands::Init => {
            init_minecraft_dir(&base_path)?;
        }
        Commands::AddAccount { username, skin, cape } => {
            init_minecraft_dir(&base_path)?;
            add_local_account(&base_path, &username, skin, cape)?;
        }
        Commands::ListAccounts => {
            list_local_accounts(&base_path)?;
        }
        Commands::Launch { username, server } => {
            init_minecraft_dir(&base_path)?;
            launch_client(&base_path, &username, &server)?;
        }
    }
    Ok(())
}

fn init_minecraft_dir(base: &Path) -> io::Result<()> {
    println!("[Launcher] Initializing official Minecraft 1.12.2 folder architecture at '{}'...", base.display());

    let dirs = [
        base.join("versions/1.12.2-rust"),
        base.join("assets/indexes"),
        base.join("assets/objects"),
        base.join("libraries"),
        base.join("skins"),
        base.join("capes"),
        base.join("resourcepacks"),
        base.join("saves"),
        base.join("logs"),
    ];

    for d in &dirs {
        fs::create_dir_all(d)?;
    }

    // 1. Create versions/1.12.2-rust/1.12.2-rust.json
    let version_json_path = base.join("versions/1.12.2-rust/1.12.2-rust.json");
    if !version_json_path.exists() {
        let version_manifest = serde_json::json!({
            "id": "1.12.2-rust",
            "time": "2026-10-11T12:00:00+00:00",
            "releaseTime": "2026-10-11T12:00:00+00:00",
            "type": "release",
            "mainClass": "net.minecraft.client.main.Main",
            "minecraftArguments": "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type}",
            "minimumLauncherVersion": 18,
            "assets": "1.12",
            "assetIndex": {
                "id": "1.12",
                "sha1": "041c2c31ff710ef50d75c879d76c9ad0ba0119e6",
                "size": 174627,
                "totalSize": 164096057,
                "url": "https://launchermeta.mojang.com/v1/packages/041c2c31ff710ef50d75c879d76c9ad0ba0119e6/1.12.json"
            },
            "downloads": {
                "client": {
                    "sha1": "0f275bc1547d01fa5f56ba34bdc87d981ee12daf",
                    "size": 10180113,
                    "url": "https://piston-data.mojang.com/v1/objects/0f275bc1547d01fa5f56ba34bdc87d981ee12daf/client.jar"
                }
            }
        });
        let mut f = File::create(&version_json_path)?;
        f.write_all(serde_json::to_string_pretty(&version_manifest)?.as_bytes())?;
        println!("[Launcher] Created version manifest: {}", version_json_path.display());
    }

    // 2. Create launcher_profiles.json
    let profiles_path = base.join("launcher_profiles.json");
    if !profiles_path.exists() {
        let initial_profiles = LauncherProfiles {
            profiles: HashMap::new(),
            selected_user: None,
            auth_database: HashMap::new(),
            client_token: "rminecraft-local-token-2026".to_string(),
        };
        let mut f = File::create(&profiles_path)?;
        f.write_all(serde_json::to_string_pretty(&initial_profiles)?.as_bytes())?;
        println!("[Launcher] Created launcher_profiles.json: {}", profiles_path.display());
    }

    println!("[Launcher] Architecture setup complete!");
    Ok(())
}

fn add_local_account(base: &Path, username: &str, skin: Option<String>, cape: Option<String>) -> io::Result<()> {
    let profiles_path = base.join("launcher_profiles.json");
    let content = fs::read_to_string(&profiles_path)?;
    let mut profiles_data: LauncherProfiles = serde_json::from_str(&content).unwrap_or(LauncherProfiles {
        profiles: HashMap::new(),
        selected_user: None,
        auth_database: HashMap::new(),
        client_token: "rminecraft-local-token-2026".to_string(),
    });

    let uuid = offline_player_uuid(username);
    let uuid_str = uuid.hyphenated().to_string();

    // Copy skin if provided
    let final_skin_path = if let Some(skin_file) = skin {
        let dest = base.join(format!("skins/{}.png", username));
        if Path::new(&skin_file).exists() {
            fs::copy(&skin_file, &dest)?;
            println!("[Launcher] Copied skin file to {}", dest.display());
            Some(dest.to_string_lossy().to_string())
        } else {
            println!("[Launcher] Warning: Skin file {} not found", skin_file);
            None
        }
    } else {
        None
    };

    // Copy cape if provided
    let final_cape_path = if let Some(cape_file) = cape {
        let dest = base.join(format!("capes/{}.png", username));
        if Path::new(&cape_file).exists() {
            fs::copy(&cape_file, &dest)?;
            println!("[Launcher] Copied cape file to {}", dest.display());
            Some(dest.to_string_lossy().to_string())
        } else {
            println!("[Launcher] Warning: Cape file {} not found", cape_file);
            None
        }
    } else {
        None
    };

    // Add to auth database
    profiles_data.auth_database.insert(uuid_str.clone(), AuthUser {
        display_name: username.to_string(),
        access_token: "local-offline-token".to_string(),
        username: username.to_string(),
        uuid: uuid_str.clone(),
    });

    // Add profile
    profiles_data.profiles.insert(format!("Profile_{}", username), Profile {
        name: format!("rMinecraft ({})", username),
        last_version_id: "1.12.2-rust".to_string(),
        created: "2026-10-11T12:00:00Z".to_string(),
        profile_type: "custom".to_string(),
        skin_path: final_skin_path,
        cape_path: final_cape_path,
    });

    // Set selected user
    profiles_data.selected_user = Some(SelectedUser {
        account: uuid_str.clone(),
        profile: format!("Profile_{}", username),
    });

    let mut f = File::create(&profiles_path)?;
    f.write_all(serde_json::to_string_pretty(&profiles_data)?.as_bytes())?;

    println!("[Launcher] Local Account '{}' registered successfully with UUID: {}", username, uuid_str);
    Ok(())
}

fn list_local_accounts(base: &Path) -> io::Result<()> {
    let profiles_path = base.join("launcher_profiles.json");
    if !profiles_path.exists() {
        println!("[Launcher] No accounts configured yet.");
        return Ok(());
    }

    let content = fs::read_to_string(&profiles_path)?;
    let profiles_data: LauncherProfiles = serde_json::from_str(&content)?;

    println!("============================================================");
    println!("  rMinecraft Launcher - Registered Offline Local Accounts");
    println!("============================================================");
    for (uuid, auth) in &profiles_data.auth_database {
        println!("  • User: {:<16} UUID: {}", auth.username, uuid);
        if let Some(prof) = profiles_data.profiles.get(&format!("Profile_{}", auth.username)) {
            if let Some(ref s) = prof.skin_path {
                println!("    └─ Skin: {}", s);
            }
            if let Some(ref c) = prof.cape_path {
                println!("    └─ Cape: {}", c);
            }
        }
    }
    println!("============================================================");
    Ok(())
}

fn launch_client(base: &Path, username: &str, server: &str) -> io::Result<()> {
    println!("[Launcher] Preparing to launch rMinecraft 1.12.2 for player '{}'...", username);

    let profiles_path = base.join("launcher_profiles.json");
    let mut skin_arg = None;
    let mut cape_arg = None;

    if profiles_path.exists() {
        if let Ok(content) = fs::read_to_string(&profiles_path) {
            if let Ok(data) = serde_json::from_str::<LauncherProfiles>(&content) {
                if let Some(prof) = data.profiles.get(&format!("Profile_{}", username)) {
                    skin_arg = prof.skin_path.clone();
                    cape_arg = prof.cape_path.clone();
                }
            }
        }
    }

    let uuid = offline_player_uuid(username);
    let uuid_str = uuid.hyphenated().to_string();

    // Look for client binary
    let client_candidates = [
        PathBuf::from("/home/user/rMinecraft/target/release/rminecraft-client"),
        PathBuf::from("/home/user/rMinecraft/target/debug/rminecraft-client"),
        base.join("versions/1.12.2-rust/rminecraft-client"),
    ];

    let client_bin = client_candidates.iter().find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| PathBuf::from("cargo"));

    println!("[Launcher] Executing client binary: {}", client_bin.display());

    let mut cmd = if client_bin == PathBuf::from("cargo") {
        let mut c = Command::new("cargo");
        c.args(["run", "-p", "rminecraft-client", "--"]);
        c
    } else {
        Command::new(client_bin)
    };

    cmd.arg("--username").arg(username)
       .arg("--uuid").arg(&uuid_str)
       .arg("--version").arg("1.12.2-rust")
       .arg("--gameDir").arg(base.to_string_lossy().to_string())
       .arg("--assetsDir").arg(base.join("assets").to_string_lossy().to_string())
       .arg("--server").arg(server);

    if let Some(s) = skin_arg {
        cmd.arg("--skin").arg(s);
    }
    if let Some(c) = cape_arg {
        cmd.arg("--cape").arg(c);
    }

    println!("[Launcher] Spawning game process: {:?}", cmd);
    let mut child = cmd.spawn()?;
    let status = child.wait()?;

    println!("[Launcher] Client process exited with status: {}", status);
    Ok(())
}
