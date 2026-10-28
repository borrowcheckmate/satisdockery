use clap::Args;

#[allow(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent command line switch"
)]
#[derive(Debug, Args)]
pub struct ServerSettings {
    /// Amount of max players
    #[arg(long, default_value_t = 4)]
    pub players: u32,
    /// Port where the gameserver is exposed (UDP)
    #[arg(long, default_value_t = 7777)]
    pub game_port: u16,
    /// Port where the messaging port is exposed (TCP)
    #[arg(long, default_value_t = 8888)]
    pub messaging_port: u16,
    /// Amount of seconds between each autosave
    #[arg(long, default_value_t = 300)]
    pub autosave_interval: u32,
    /// Max autosaves value that will be saved
    #[arg(long, default_value_t = 5)]
    pub max_autosaves: u32,
    /// Max objects that may exist in the world
    #[arg(long, default_value_t = 2_162_688)]
    pub max_objects: u32,
    /// Max tick rate (amount of times the states are calculated per second)
    #[arg(long, default_value_t = 30)]
    pub max_tickrate: u32,
    /// Enable server streaming (so the client only gets a portion of the world thus reducing memory usage)
    #[arg(long)]
    pub enable_serverstreaming: bool,
    /// Max timeout duration until a player is kicked
    #[arg(long, default_value_t = 10)]
    pub max_timeout: u32,
    /// Disable seasonal events like ficmas (whyy???!)
    #[arg(long)]
    pub disable_seasonalevents: bool,
    /// Add Unreal arguments for the dedicated server if you want to do anything more besides the default args
    #[arg(long, value_delimiter = ',', allow_hyphen_values = true)]
    pub additional_dedicatedserver_args: Vec<String>,
}
