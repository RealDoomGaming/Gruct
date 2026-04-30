use std::{
    env::var,
    sync::OnceLock,
    net::TcpStream,
    error::Error,
};
use crate::response::send_back;

// constants
pub const REPOS_DIR: &str = "/var/lib/gruct-repos";
pub const _LOGS_DIR: &str = "/var/log/gruct-logs";
pub const GIT_KEYS_DIR: &str = "/var/lib/gruct/git-keys";
// end

// env variable
pub static PASSWORD: OnceLock<String> = OnceLock::new();
// end

pub fn get_password() -> &'static str {
    PASSWORD.get_or_init(|| {
        var("PASSWORD_ENV").expect("PASSWORD_ENV must be set")
    })
}
pub fn compare_passwd(passwd: &str, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    if passwd != get_password() {
        let message = "Authentication failed";
        send_back(message, &stream, 401);
        return Err("Authentication failed".into());
    }

    return Ok(());
}
