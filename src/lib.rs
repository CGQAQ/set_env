//! This crate allows you to permanently set environment variables
//!
//! # Examples
//! ```rust
//! // Check if DUMMY is set, if not set it to 1
//! // export DUMMY=1
//! env_perm::check_or_set("DUMMY", 1).expect("Failed to find or set DUMMY");
//! // Append $HOME/some/cool/bin to $PATH
//! // export PATH= "$HOME/some/cool/bin:$PATH"
//! env_perm::append("PATH", "$HOME/some/cool/bin").expect("Couldn't find PATH");
//! // Sets a variable without checking if it exists.
//! // Note you need to use a raw string literal to include ""
//! // export DUMMY="/something"
//! env_perm::set("DUMMY", r#""/something""#).expect("Failed to set DUMMY");
//! ```

#[cfg(target_family = "unix")]
use dirs;
#[cfg(target_family = "unix")]
use std::fs::{File, OpenOptions};
#[cfg(target_family = "unix")]
use std::io::Write;
#[cfg(target_family = "unix")]
use std::path::PathBuf;

#[cfg(target_family = "windows")]
use winreg::RegKey;

#[cfg(target_family = "windows")]
use winreg::enums::HKEY_CURRENT_USER;

#[cfg(target_family = "unix")]
use std::env;
#[cfg(target_family = "unix")]
use std::env::VarError;
use std::fmt;
use std::io;

#[cfg(target_family="windows")]
pub fn do_prerequisites() {
    use std::fs;

    let path = dirs::document_dir();
    if let Some(path) = path {
        let template = include_str!("../scripts/profile.ps1");
        let path = path.join("WindowsPowerShell");
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }
        let path = path.join("Profile.ps1");
        if !path.exists() {
            fs::write(path, template).unwrap();
        } else {
            let mut content = fs::read_to_string(path).unwrap();
            if !content.contains("# ----------------------------------SET_ENV_BEG") || !content.contains("# ----------------------------------SET_ENV_END") {
                content.push('\r');
                content.push('\n');
                content.push_str(template);
            }
        }
        return
    }

    eprintln!("document path is not exists");
    std::process::exit(1);
}

#[cfg(target_os = "windows")]
pub fn inject(it: &str) -> io::Result<()>{
    use std::fs;

    do_prerequisites();

    let profile_path = dirs::document_dir().unwrap().join("WindowsPowerShell/Profile.ps1");

    let content = fs::read_to_string(&profile_path)?;
    let mut content_parts: Vec<&str> = content.split("\r\n").collect();

    let idx = content_parts.iter().position(|it| it == &"# ----------------------------------SET_ENV_DEFS_END").unwrap();
    content_parts.insert(idx, it);

    fs::write(profile_path,  content_parts.join("\r\n"))
}


/// Checks if a environment variable is set.
/// If it is then nothing will happen.
/// If it's not then it will be added
/// to your profile.
#[cfg(target_family = "unix")]
pub fn check_or_set<T, U>(var: T, value: U) -> io::Result<()>
where
    T: fmt::Display + AsRef<std::ffi::OsStr>,
    U: fmt::Display,
{
    env::var(&var).map(|_| ()).or_else(|_| set(var, value))
}

#[cfg(target_os = "windows")]
pub fn check_or_set<T, U>(var: T, value: U) -> io::Result<()>
where
    T: fmt::Display + AsRef<std::ffi::OsStr>,
    U: fmt::Display,
{
    inject(format!("setenv_set_if_not_exist {} {}", var, value).as_str())
}

#[cfg(target_family = "unix")]
pub fn get<'a, T: fmt::Display>(var: T) -> io::Result<String> {
    env::var(var.to_string()).map_err(|err| match err {
        VarError::NotPresent => {
            io::Error::new(io::ErrorKind::NotFound, "Variable not present.")
        }
        VarError::NotUnicode(_) => {
            io::Error::new(io::ErrorKind::Unsupported, "Encoding not supported.")
        }
    })
}

#[cfg(target_os = "windows")]
pub fn get<'a, T: fmt::Display>(var: T) -> io::Result<String> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Environment")?;
    Ok(key.get_value::<String, String>(var.to_string())?.to_string())
}

/// Appends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_family = "unix")]
pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"{}:${}\"", var, value, var)?;
    profile.flush()
}
/// Appends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_os = "windows")]
pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    inject(format!("setenv_append {} {}", var, value).as_str())
}

/// Prepends a value to an environment variable
/// Useful for prepending a value to PATH
#[cfg(target_family = "unix")]
pub fn prepend<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"${}:{}\"", var, value, var)?;
    profile.flush()
}

/// Prepends a value to an environment variable
/// Useful for prepending a value to PATH
#[cfg(target_os = "windows")]
pub fn prepend<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    inject(format!("setenv_prepend {} {}", var, value).as_str())
}

/// Sets an environment variable without checking
/// if it exists.
/// If it does you will end up with two
/// assignments in your profile.
/// It's recommended to use `check_or_set`
/// unless you are certain it doesn't exist.
#[cfg(target_family = "unix")]
pub fn set<T: fmt::Display, U: fmt::Display>(var: T, value: U) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}={}", var, value)?;
    profile.flush()
}
/// Sets an environment variable without checking
/// if it exists.
/// If it does you will override the value.
#[cfg(target_os = "windows")]
pub fn set<T: fmt::Display, U: fmt::Display>(var: T, value: U) -> io::Result<()> {
    inject(format!("setenv_set {} {}", var, value).as_str())?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn get_profile() -> io::Result<File> {
    dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No home directory"))
        .and_then(find_profile)
}

#[cfg(target_family = "unix")]
fn find_profile(mut profile: PathBuf) -> io::Result<File> {
    profile.push(".bash_profile");
    let mut oo = OpenOptions::new();
    oo.append(true).create(false);
    oo.open(profile.clone())
        .or_else(|_| {
            profile.pop();
            profile.push(".bash_login");
            oo.open(profile.clone())
        })
        .or_else(|_| {
            profile.pop();
            profile.push(".profile");
            oo.open(profile.clone())
        })
        .or_else(|_| {
            profile.pop();
            profile.push(".bash_profile");
            oo.create(true);
            oo.open(profile.clone())
        })
}
