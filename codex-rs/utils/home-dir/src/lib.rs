use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::env;
use std::path::PathBuf;

/// Returns the path to the CLI configuration directory.
///
/// Anzoth binaries use `ANZOTH_HOME` when it is set and otherwise default to
/// `~/.anzoth`. Legacy Codex binaries continue to use `CODEX_HOME` and default
/// to `~/.codex`.
///
/// - If the relevant home environment variable is set, the value must exist and
///   be a directory. The value will be canonicalized and this function will Err
///   otherwise.
/// - If the relevant environment variable is not set, this function does not
///   verify that the directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let home_kind = home_kind_for_executable_stem(current_exe_stem().as_deref());
    let home_env = env::var(home_kind.env_var())
        .ok()
        .filter(|val| !val.is_empty());
    find_home_from_env(home_kind, home_env.as_deref())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HomeKind {
    Codex,
    Anzoth,
}

impl HomeKind {
    fn env_var(self) -> &'static str {
        match self {
            Self::Codex => "CODEX_HOME",
            Self::Anzoth => "ANZOTH_HOME",
        }
    }

    fn default_suffix(self) -> &'static str {
        match self {
            Self::Codex => ".codex",
            Self::Anzoth => ".anzoth",
        }
    }
}

fn current_exe_stem() -> Option<String> {
    let exe_stem = env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| stem.to_owned()))
        .and_then(|stem| stem.to_str().map(|stem| stem.to_ascii_lowercase()));
    exe_stem
}

fn home_kind_for_executable_stem(exe_stem: Option<&str>) -> HomeKind {
    match exe_stem {
        Some(stem) if stem.starts_with("anzoth") => HomeKind::Anzoth,
        _ => HomeKind::Codex,
    }
}

fn find_home_from_env(
    home_kind: HomeKind,
    home_env: Option<&str>,
) -> std::io::Result<AbsolutePathBuf> {
    // Honor the relevant home environment variable when it is set to allow
    // users (and tests) to override the default location.
    match home_env {
        Some(val) => {
            let path = PathBuf::from(val);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "{} points to {val:?}, but that path does not exist",
                        home_kind.env_var()
                    ),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read {} {val:?}: {err}", home_kind.env_var()),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "{} points to {val:?}, but that path is not a directory",
                        home_kind.env_var()
                    ),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!(
                            "failed to canonicalize {} {val:?}: {err}",
                            home_kind.env_var()
                        ),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(home_kind.default_suffix());
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HomeKind;
    use super::find_home_from_env;
    use super::home_kind_for_executable_stem;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::env;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err =
            find_home_from_env(HomeKind::Codex, Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_home_from_env(HomeKind::Codex, Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved =
            find_home_from_env(HomeKind::Codex, Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_default_home_dir() {
        let resolved =
            find_home_from_env(/*home_kind*/ HomeKind::Codex, /*home_env*/ None)
                .expect("default CODEX_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".codex");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_anzoth_home_without_env_uses_default_home_dir() {
        let resolved =
            find_home_from_env(/*home_kind*/ HomeKind::Anzoth, /*home_env*/ None)
                .expect("default ANZOTH_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".anzoth");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_anzoth_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp anzoth home path should be valid utf-8");

        let resolved =
            find_home_from_env(HomeKind::Anzoth, Some(temp_str)).expect("valid ANZOTH_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn anzoth_executable_prefers_anzoth_home_and_ignores_codex_home() {
        let temp_home = TempDir::new().expect("temp home");
        let anzoth_home = temp_home.path().join("anzoth-home");
        let codex_home = temp_home.path().join(".codex-home");
        fs::create_dir_all(&anzoth_home).expect("create ANZOTH_HOME");
        fs::create_dir_all(&codex_home).expect("create CODEX_HOME");

        let previous_anzoth_home = env::var_os("ANZOTH_HOME");
        let previous_codex_home = env::var_os("CODEX_HOME");
        unsafe {
            env::remove_var("ANZOTH_HOME");
            env::set_var("CODEX_HOME", &codex_home);
        }

        let resolved =
            resolve_home_for_executable_stem(Some("anzoth")).expect("resolve anzoth home");
        let mut expected = home_dir().expect("home dir");
        expected.push(".anzoth");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);

        unsafe {
            if let Some(previous_anzoth_home) = previous_anzoth_home {
                env::set_var("ANZOTH_HOME", previous_anzoth_home);
            } else {
                env::remove_var("ANZOTH_HOME");
            }

            if let Some(previous_codex_home) = previous_codex_home {
                env::set_var("CODEX_HOME", previous_codex_home);
            } else {
                env::remove_var("CODEX_HOME");
            }
        }
    }

    #[test]
    fn codex_executable_prefers_codex_home() {
        let temp_home = TempDir::new().expect("temp home");
        let codex_home = temp_home.path().join(".codex-home");
        fs::create_dir_all(&codex_home).expect("create CODEX_HOME");

        let previous_anzoth_home = env::var_os("ANZOTH_HOME");
        let previous_codex_home = env::var_os("CODEX_HOME");
        unsafe {
            env::remove_var("ANZOTH_HOME");
            env::set_var("CODEX_HOME", &codex_home);
        }

        let resolved = resolve_home_for_executable_stem(Some("codex")).expect("resolve codex home");
        let expected = codex_home.canonicalize().expect("canonicalize codex home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);

        unsafe {
            if let Some(previous_anzoth_home) = previous_anzoth_home {
                env::set_var("ANZOTH_HOME", previous_anzoth_home);
            } else {
                env::remove_var("ANZOTH_HOME");
            }

            if let Some(previous_codex_home) = previous_codex_home {
                env::set_var("CODEX_HOME", previous_codex_home);
            } else {
                env::remove_var("CODEX_HOME");
            }
        }
    }

    fn resolve_home_for_executable_stem(
        exe_stem: Option<&str>,
    ) -> std::io::Result<AbsolutePathBuf> {
        let home_kind = home_kind_for_executable_stem(exe_stem);
        let home_env = env::var(home_kind.env_var())
            .ok()
            .filter(|val| !val.is_empty());
        find_home_from_env(home_kind, home_env.as_deref())
    }
}
