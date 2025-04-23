use clap::Parser;

const DEFAULT_PDS: &'static str = "https://bsky.app";

#[derive(Parser, Debug, Clone)]
pub struct LoginData {
    /// Username of the user to log onto the PDS.
    #[arg(long, env = "ATPAGE_USERNAME")]
    pub username: String,

    /// Password for the user to log onto the PDS.
    #[arg(long, env = "ATPAGE_PASSWORD")]
    pub password: String,

    /// PDS to log onto.
    #[arg(long, env, default_value = DEFAULT_PDS, env = "ATPAGE_PDS")]
    pub pds: String,
}

#[derive(Parser, Debug)]
#[command(version, about)]
/// Publishes HTML websites under the industries.geesawra.website collection, for a given user and PDS.
pub enum Command {
    /// Post a new industries.geesawra.website on the configured PDS for the logged-in user.
    Post {
        #[command(flatten)]
        login_data: LoginData,

        /// Directory containing the website to upload to the PDS.
        #[arg(long, env = "ATPAGE_SRC")]
        src: String,

        /// File containing HTML OpenGraph tags.
        #[arg(long)]
        opengraph_path: Option<String>,
    },

    /// Deletes the industries.geesawra.website from the configured PDS for the logged-in user.
    Nuke(LoginData),

    /// Compiles atpage_renderer, useful for local development.
    Compile {
        /// AT URI containing a industries.geesawra.website collection.
        at_uri: String,

        /// File containing HTML OpenGraph tags.
        #[arg(long)]
        opengraph_path: Option<String>,
    },
}
