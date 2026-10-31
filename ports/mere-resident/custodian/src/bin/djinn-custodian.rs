// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custodian process. The service manager starts it with a fixed
//! configuration path and hands it its listening socket (systemd on Linux,
//! launchd on macOS) or it creates its named pipe (Windows SCM). It reads
//! the configuration, holds the vault locked, applies this OS's confinement
//! and only then accepts a connection. `--socket` binds a path itself and
//! is refused outside development mode.
//!
//!   djinn-custodian --config PATH [--socket PATH]
//!   djinn-custodian init  --config PATH      (passphrase twice on stdin)
//!   djinn-custodian adopt --config PATH [--from VAULT-DIR] [--data-from DATA-ROOT]
//!       --from: a passphrase vault (its passphrase on stdin) or an OS-rooted
//!       Windows vault (the new passphrase twice, run as the owner);
//!       --data-from: the wallet and credential catalog, no passphrase
//!   djinn-custodian wallet-files --data-root DIR   (the wallet's files, NUL-separated)
//!   djinn-custodian consent --client PATH   (Windows: the logon helper that
//!       puts petitions to the owner through Windows Hello)

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use djinn_custodian::adoption::{self, VaultKind};
use djinn_custodian::consent::Mechanism;
use djinn_custodian::core::Core;
use djinn_custodian::driver::Service;
use djinn_custodian::platform::{self, Confinement};
use djinn_custodian::policy::{Config, ConsentMode};
use zeroize::Zeroizing;

fn main() {
    if let Err(error) = run() {
        eprintln!("djinn-custodian: {error}");
        std::process::exit(2);
    }
}

struct Args {
    command: String,
    config: Option<PathBuf>,
    client: Option<PathBuf>,
    socket: Option<PathBuf>,
    from: Option<PathBuf>,
    data_from: Option<PathBuf>,
}

impl Args {
    fn config(&self) -> Result<&Path, String> {
        self.config.as_deref().ok_or_else(|| "--config is required".to_string())
    }
}

fn parse() -> Result<Args, String> {
    let mut raw = std::env::args().skip(1).peekable();
    let command = match raw.peek().map(String::as_str) {
        Some("init") | Some("adopt") | Some("consent") => raw.next().expect("peeked"),
        _ => "serve".to_string(),
    };
    let (mut config, mut client, mut socket, mut from, mut data_from) = (None, None, None, None, None);
    while let Some(flag) = raw.next() {
        let value = raw.next().ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--config" => config = Some(PathBuf::from(value)),
            "--client" => client = Some(PathBuf::from(value)),
            "--socket" => socket = Some(PathBuf::from(value)),
            "--from" => from = Some(PathBuf::from(value)),
            "--data-from" => data_from = Some(PathBuf::from(value)),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    for (flag, path) in [("--config", &config), ("--client", &client)] {
        if path.as_ref().is_some_and(|path| !path.is_absolute()) {
            return Err(format!("{flag} must be absolute"));
        }
    }
    Ok(Args {
        command,
        config,
        client,
        socket,
        from,
        data_from,
    })
}

fn run() -> Result<(), String> {
    if std::env::args().nth(1).as_deref() == Some("wallet-files") {
        return wallet_files();
    }
    let args = parse()?;
    if args.command == "consent" {
        return consent(args.client.as_deref().ok_or("consent needs --client")?);
    }
    let path = args.config()?;
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let config = Config::parse(&bytes).map_err(|e| e.to_string())?;
    match args.command.as_str() {
        "init" => init(&config),
        "adopt" => adopt(&config, args.from.as_deref(), args.data_from.as_deref()),
        _ => serve(config, &args),
    }
}

fn serve(config: Config, args: &Args) -> Result<(), String> {
    let development = config.development;
    if args.socket.is_some() && !development {
        return Err("--socket is for development; a protected custodian takes its listener from the service manager".into());
    }
    let plan = Confinement {
        state: config.vault.directory.clone(),
        config: args.config()?.to_path_buf(),
        required: !development,
    };
    let consent = config.consent;
    let core = Core::start(config.clone()).map_err(|r| format!("start: {r}"))?;
    let (listener, agent) = listeners(&config, args)?;
    let mechanism = mechanism(consent)?;
    // Every thread starts after confinement: Landlock and seccomp bind the
    // calling thread and its descendants only.
    let applied = platform::confine(&plan)?;
    for note in &applied.notes {
        eprintln!("djinn-custodian: {note}");
    }
    eprintln!("djinn-custodian: serving instance {} (locked)", core.instance());
    let service = Service::new(core, mechanism);
    #[cfg(unix)]
    if let Some(agent) = agent {
        djinn_custodian::agent::spawn(service.clone(), agent, config.owner.unix_uid).map_err(|e| format!("agent: {e}"))?;
        eprintln!("djinn-custodian: serving the SSH agent");
    }
    #[cfg(windows)]
    {
        let owner = config.owner.windows_sid.clone().ok_or("owner.windows_sid is required on Windows")?;
        djinn_custodian::agent_pipe::spawn(service.clone(), agent, owner).map_err(|e| format!("agent: {e}"))?;
        eprintln!("djinn-custodian: serving the SSH agent");
    }
    #[cfg(windows)]
    {
        let stopping = service.clone();
        let serving = service.clone();
        return platform::windows::run_service(
            Box::new(move |_| {
                let _ = serving.serve(&*listener);
            }),
            Box::new(move || stopping.lock_now()),
        );
    }
    #[cfg(not(windows))]
    service.serve(&*listener).map_err(|e| format!("serve: {e}"))
}

#[cfg(unix)]
type AgentListener = std::os::unix::net::UnixListener;
/// The agent's pipe name.
#[cfg(windows)]
type AgentListener = String;

#[cfg(unix)]
type AgentEndpoint = Option<AgentListener>;
#[cfg(windows)]
type AgentEndpoint = AgentListener;

/// The custody listener and the SSH agent's: where configured on Unix, always
/// on Windows.
fn listeners(config: &Config, args: &Args) -> Result<(Box<dyn custody_wire::transport::Listener>, AgentEndpoint), String> {
    #[cfg(unix)]
    {
        use custody_wire::transport::UnixSocketListener;
        // Signed principals are identified by code signature where the OS
        // can say so (macOS); elsewhere they never match.
        let signed = config.signed_identities();
        if let Some(path) = &args.socket {
            let custody = UnixSocketListener::bind(path).map_err(|e| format!("bind {}: {e}", path.display()))?;
            let agent = match &config.vault.agent_endpoint {
                Some(endpoint) => Some(bind_agent(std::path::Path::new(endpoint))?),
                None => None,
            };
            return Ok((Box::new(custody.with_signed(signed)), agent));
        }
        #[cfg(target_os = "linux")]
        {
            let mut named = UnixSocketListener::systemd_named().map_err(|e| e.to_string())?;
            if let Some(index) = named.iter().position(|(name, _)| name == "custody" || name.is_empty()) {
                let (_, custody) = named.remove(index);
                let agent = named.into_iter().find(|(name, _)| name == "agent").map(|(_, l)| l);
                return Ok((Box::new(UnixSocketListener::from_listener(custody)), agent));
            }
        }
        #[cfg(target_os = "macos")]
        if let Some(custody) = platform::macos::launchd_listener("custody").map_err(|e| e.to_string())? {
            let agent = platform::macos::launchd_listener("agent").map_err(|e| e.to_string())?;
            return Ok((Box::new(UnixSocketListener::from_listener(custody).with_signed(signed)), agent));
        }
        Err("no listening socket from the service manager".into())
    }
    #[cfg(windows)]
    {
        let _ = args;
        let owner = config.owner.windows_sid.as_deref().ok_or("owner.windows_sid is required on Windows")?;
        let name = format!(r"\\.\pipe\djinn-custodian-{owner}");
        let agent = config.vault.agent_endpoint.clone().unwrap_or_else(|| djinn_custodian::agent_pipe::default_endpoint(owner));
        Ok((Box::new(custody_wire::transport::NamedPipeListener::new(&name, owner)), agent))
    }
}

/// Bind the agent socket at 0600 (development; a protected custodian takes
/// it from the service manager).
#[cfg(unix)]
fn bind_agent(path: &std::path::Path) -> Result<std::os::unix::net::UnixListener, String> {
    use std::os::unix::fs::PermissionsExt;
    if std::fs::symlink_metadata(path).is_ok() && std::os::unix::net::UnixStream::connect(path).is_err() {
        let _ = std::fs::remove_file(path);
    }
    let listener = std::os::unix::net::UnixListener::bind(path).map_err(|e| format!("bind {}: {e}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    Ok(listener)
}

fn mechanism(consent: ConsentMode) -> Result<Option<Arc<dyn Mechanism>>, String> {
    match consent {
        ConsentMode::Polkit => {
            #[cfg(target_os = "linux")]
            return Ok(Some(Arc::new(platform::linux::PolkitHelper::fork(djinn_custodian::consent::Polkit::installed())?)));
            #[cfg(not(target_os = "linux"))]
            Err("polkit consent exists only on Linux".into())
        },
        ConsentMode::Channel | ConsentMode::Deny => Ok(None),
    }
}

fn passphrase(stdin: &mut impl BufRead) -> Result<Zeroizing<String>, String> {
    let mut line = Zeroizing::new(String::new());
    stdin.read_line(&mut line).map_err(|e| e.to_string())?;
    let keep = line.trim_end_matches(['\r', '\n']).len();
    line.truncate(keep);
    if line.is_empty() {
        return Err("an empty passphrase is not accepted".into());
    }
    Ok(line)
}

/// Create a new vault in the custodian's state directory. Refused if any
/// vault is already there: an existing identity is never replaced.
fn init(config: &Config) -> Result<(), String> {
    use castellan::custody::bootstrap::{self, PASSPHRASE_VAULT_FILE, Unlock};
    let directory = &config.vault.directory;
    if directory.join(PASSPHRASE_VAULT_FILE).exists() {
        return Err("a vault already exists here".into());
    }
    let mut stdin = std::io::stdin().lock();
    let first = passphrase(&mut stdin)?;
    let second = passphrase(&mut stdin)?;
    if first.as_bytes() != second.as_bytes() {
        return Err("the two passphrases did not match".into());
    }
    let opened = bootstrap::open_storage(directory, Unlock::passphrase(first.as_bytes().to_vec())).map_err(|e| e.to_string())?;
    let id = personae::ProfileId(config.vault.profile.clone().unwrap_or_else(|| "default".into()));
    let (profile, created) = bootstrap::load_or_create_profile(&*opened.storage, &id).map_err(|e| e.to_string())?;
    if !created {
        return Err("the new vault already held that profile".into());
    }
    println!("created profile {} with master {}", id.0, hex(&profile.master.public_key().to_bytes()));
    Ok(())
}

/// Adopt Djinn's existing vault, its wallet and credential catalog, or both.
/// The vault goes first: with `--from` it must open before any wallet moves.
fn adopt(config: &Config, from: Option<&Path>, data_from: Option<&Path>) -> Result<(), String> {
    if from.is_none() && data_from.is_none() {
        return Err("adopt needs --from, --data-from or both".into());
    }
    if let Some(from) = from {
        adopt_vault(config, from)?;
    }
    match data_from {
        Some(data_from) => adopt_data(config, data_from),
        None => Ok(()),
    }
}

/// Carry Djinn's wallet and credential catalog into the custodian's data
/// root, prove the copy is the same wallet, then remove the copies in
/// `data_from`: the seed no longer sits where Djinn can read it.
fn adopt_data(config: &Config, data_from: &Path) -> Result<(), String> {
    let data_root = config.vault.data_root.as_deref().ok_or("the configuration names no vault.data_root")?;
    let adopted = djinn_custodian::wallet::adopt_files(data_from, data_root)?;
    println!("adopted {} wallet and credential files from {} into {}", adopted.files.len(), data_from.display(), data_root.display());
    if adopted.left.is_empty() {
        return Ok(());
    }
    for (file, why) in &adopted.left {
        eprintln!("djinn-custodian: could not remove {}: {why}", data_from.join(file).display());
    }
    Err(format!("the wallet is adopted, but {} copies remain in {}; remove them as their owner", adopted.left.len(), data_from.display()))
}

/// The wallet's and catalog's files under a data root, relative and
/// NUL-separated, for an installer to stage and later remove as the owner.
fn wallet_files() -> Result<(), String> {
    use std::io::Write;
    let mut args = std::env::args().skip(2);
    let root = match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--data-root"), Some(root), None) => PathBuf::from(root),
        _ => return Err("usage: djinn-custodian wallet-files --data-root DIR".into()),
    };
    let mut out = std::io::stdout().lock();
    for file in djinn_custodian::wallet::wallet_files(&root).map_err(|e| e.to_string())? {
        out.write_all(file.to_string_lossy().as_bytes()).and_then(|()| out.write_all(&[0])).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Bring Djinn's existing vault into the state directory, then prove the
/// copy opens to the same identity. The source is left in place for the
/// owner to retire; nothing is migrated in place. A passphrase vault is
/// copied as the file it is. Djinn's OS-rooted Windows vault opens only for
/// its owner's own logon, so it is read here, as the owner, and written as a
/// new passphrase vault.
fn adopt_vault(config: &Config, from: &Path) -> Result<(), String> {
    use castellan::custody::bootstrap::PASSPHRASE_VAULT_FILE;
    let directory = &config.vault.directory;
    if directory.join(PASSPHRASE_VAULT_FILE).exists() {
        return Err("a vault already exists in the custodian's state directory".into());
    }
    let (phrase, source_master) = match adoption::kind(from)? {
        VaultKind::Passphrase => adopt_passphrase_file(directory, from)?,
        VaultKind::OsRooted => adopt_os_rooted(config, directory, from)?,
    };
    let mut adopted = djinn_custodian::host::HostCustody::locked(directory, config.vault.profile.as_deref(), None).map_err(|r| r.to_string())?;
    adopted.unlock(phrase.as_bytes()).map_err(|r| format!("open the copy: {r}"))?;
    if Some(source_master.1) != adopted.master_public_key() || Some(source_master.0.as_str()) != adopted.profile() {
        adopted.lock();
        let _ = std::fs::remove_file(directory.join(PASSPHRASE_VAULT_FILE));
        return Err("the copy did not open to the same identity; it was removed".into());
    }
    adopted.lock();
    println!(
        "adopted profile {} with master {}; retire {} once Djinn uses the custodian",
        adopted.profile().unwrap_or_default(),
        hex(&adopted.master_public_key().unwrap_or_default()),
        from.display()
    );
    Ok(())
}

/// The passphrase and the source's profile and master, after copying
/// `vault.json` and the roster choice beside it.
fn adopt_passphrase_file(directory: &Path, from: &Path) -> Result<(Zeroizing<String>, (String, [u8; 32])), String> {
    use castellan::custody::bootstrap::PASSPHRASE_VAULT_FILE;
    use castellan::custody::roster::choice_path;
    let source = from.join(PASSPHRASE_VAULT_FILE);
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let staged = directory.join(format!("{PASSPHRASE_VAULT_FILE}.adopting"));
    std::fs::copy(&source, &staged).map_err(|e| e.to_string())?;
    std::fs::File::open(&staged).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
    if choice_path(from).is_file() {
        std::fs::copy(choice_path(from), choice_path(directory)).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&staged, directory.join(PASSPHRASE_VAULT_FILE)).map_err(|e| e.to_string())?;
    let phrase = passphrase(&mut std::io::stdin().lock())?;
    // The source is only read: no lock marker is written or cleared there.
    use castellan::custody::bootstrap::{self, Unlock};
    use personae::IdentityProvider;
    let opened = bootstrap::open_storage(from, Unlock::passphrase(phrase.as_bytes().to_vec())).map_err(|e| format!("open the source: {e}"))?;
    let id = castellan::custody::roster::resolve_profile(&*opened.storage, from).map_err(|e| e.to_string())?;
    let vault = castellan::custody::IdentityVault::open(opened.storage, &id).map_err(|e| e.to_string())?;
    Ok((phrase, (id.0.clone(), vault.master_public_key().to_bytes())))
}

/// Only the owner's own logon opens an OS-rooted vault, so this refuses any
/// other account, then writes a new passphrase vault from it.
fn adopt_os_rooted(config: &Config, directory: &Path, from: &Path) -> Result<(Zeroizing<String>, (String, [u8; 32])), String> {
    require_owner(config)?;
    let first = passphrase(&mut std::io::stdin().lock())?;
    let second = passphrase(&mut std::io::stdin().lock())?;
    if first.as_bytes() != second.as_bytes() {
        return Err("the two passphrases did not match".into());
    }
    let source = adoption::open_os_rooted(from)?;
    let id = castellan::custody::roster::resolve_profile(&source, from).map_err(|e| e.to_string())?;
    let master = {
        use castellan::custody::IdentityStorage;
        source.load_profile(&id).map_err(|e| format!("read profile {}: {e}", id.0))?.master.public_key().to_bytes()
    };
    adoption::into_passphrase_vault(&source, from, directory, first.as_bytes())?;
    Ok((first, (id.0.clone(), master)))
}

#[cfg(windows)]
fn require_owner(config: &Config) -> Result<(), String> {
    let owner = config.owner.windows_sid.as_deref().ok_or("owner.windows_sid is required on Windows")?;
    let current = custody_wire::peer::current_user_sid().map_err(|e| e.to_string())?;
    if current != owner {
        return Err(format!("run adopt as the owner ({owner}); this user is {current}"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn require_owner(_: &Config) -> Result<(), String> {
    Err("an OS-rooted Djinn vault can be adopted only on Windows, as its owner".into())
}

/// The Windows logon helper: put each petition to the owner through
/// Windows Hello until the process is ended.
#[cfg(windows)]
fn consent(client_file: &Path) -> Result<(), String> {
    use custody_wire::Client;
    use custody_wire::transport::NamedPipeConnector;
    use djinn_custodian::helper::{self, ClientFile, Retry};
    use djinn_custodian::platform::hello::Hello;
    detach_own_console();
    let file = ClientFile::read(client_file)?;
    let hello = Hello::new()?;
    let connector = NamedPipeConnector::new(&file.endpoint);
    let stop = std::sync::atomic::AtomicBool::new(false);
    helper::run(&|| Client::open(&connector, &file.server), &hello, &stop, Retry::default(), &|text| eprintln!("djinn-custodian consent: {text}"));
    Ok(())
}

/// The logon task starts this console program with a console of its own,
/// which would sit open for the whole session; one started from a shell
/// keeps the shell's console and its messages.
#[cfg(windows)]
fn detach_own_console() {
    use windows_sys::Win32::System::Console::{FreeConsole, GetConsoleProcessList};
    let mut attached = [0u32; 2];
    // SAFETY: a buffer of the length passed; FreeConsole takes nothing.
    unsafe {
        if GetConsoleProcessList(attached.as_mut_ptr(), 2) == 1 {
            FreeConsole();
        }
    }
}

#[cfg(not(windows))]
fn consent(_: &Path) -> Result<(), String> {
    Err("the consent helper is the Windows logon helper; this platform's consent is polkit or the signed host".into())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
