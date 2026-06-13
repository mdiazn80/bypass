use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bypass_core::{ResolutionSource, Vault};

/// Opens the credential vault, mapping backend failures to an actionable error.
pub fn open_vault() -> Result<Vault> {
    Vault::new().map_err(|e| {
        anyhow!(
            "could not open credential vault: {e}\n\
             On Linux without a running Secret Service, set BYPASS_MASTER_KEY \
             to a base64-encoded 32-byte key (see the README)."
        )
    })
}

/// `bypass use <context>`
pub fn use_context(name: &str) -> Result<()> {
    let vault = open_vault()?;
    vault
        .set_active(Some(name))
        .with_context(|| format!("could not activate context '{name}'"))?;
    println!("Active context set to '{name}'.");
    Ok(())
}

/// `bypass list`
pub fn list() -> Result<()> {
    let vault = open_vault()?;
    let contexts = vault.list_contexts()?;
    let cwd = env::current_dir()?;
    let resolved = vault.resolve(&cwd)?;
    let active = resolved.name.clone();

    if contexts.is_empty() {
        println!("No credential contexts yet. Create one in the Bypass app.");
        return Ok(());
    }

    for ctx in &contexts {
        let marker = if Some(&ctx.name) == active.as_ref() { "*" } else { " " };
        let count = vault.list_keys(&ctx.name).map(|k| k.len()).unwrap_or(0);
        let desc = if ctx.description.is_empty() {
            String::new()
        } else {
            format!(" - {}", ctx.description)
        };
        println!("{marker} {} ({} vars){}", ctx.name, count, desc);
    }

    match resolved.source {
        ResolutionSource::ProjectFile(path) => {
            println!("\nActive: {} (from {})", active.unwrap_or_default(), path.display());
        }
        ResolutionSource::Global => {
            println!("\nActive: {} (global)", active.unwrap_or_default());
        }
        ResolutionSource::None => println!("\nActive: none"),
    }
    Ok(())
}

/// `bypass run -- <command...>`
pub fn run(command: &[String]) -> Result<()> {
    let vault = open_vault()?;
    let cwd = env::current_dir()?;
    let (resolved, vars) = vault.resolved_vars(&cwd)?;

    if resolved.name.is_none() {
        eprintln!("bypass: warning: no active context; running without injected variables");
    }

    let (program, args) = command
        .split_first()
        .ok_or_else(|| anyhow!("no command provided"))?;

    let status = Command::new(program)
        .args(args)
        .envs(&vars)
        .status()
        .with_context(|| format!("failed to run '{program}'"))?;

    std::process::exit(status.code().unwrap_or(1));
}

/// `bypass shell`
pub fn shell() -> Result<()> {
    let vault = open_vault()?;
    let cwd = env::current_dir()?;
    let (resolved, vars) = vault.resolved_vars(&cwd)?;
    let name = resolved.name.unwrap_or_else(|| "none".to_string());

    let shell = if cfg!(windows) {
        env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    } else {
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    };

    eprintln!(
        "bypass: launching shell with context '{name}' ({} vars). Type 'exit' to return.",
        vars.len()
    );

    let status = Command::new(&shell)
        .envs(&vars)
        .env("BYPASS_CONTEXT", &name)
        .status()
        .with_context(|| format!("failed to launch shell '{shell}'"))?;

    std::process::exit(status.code().unwrap_or(0));
}

/// Reads the export passphrase from the environment to keep it out of argv and
/// shell history.
fn export_passphrase() -> Result<String> {
    let pass = env::var("BYPASS_EXPORT_PASSPHRASE").map_err(|_| {
        anyhow!("set BYPASS_EXPORT_PASSPHRASE to the passphrase used to encrypt/decrypt the export")
    })?;
    if pass.is_empty() {
        return Err(anyhow!("BYPASS_EXPORT_PASSPHRASE must not be empty"));
    }
    Ok(pass)
}

/// `bypass export <file>`
pub fn export(file: &Path) -> Result<()> {
    let passphrase = export_passphrase()?;
    let vault = open_vault()?;
    vault.export(file, &passphrase)?;
    println!("Exported encrypted vault to {}", file.display());
    Ok(())
}

/// `bypass import <file>`
pub fn import(file: &Path) -> Result<()> {
    let passphrase = export_passphrase()?;
    let vault = open_vault()?;
    vault.import(file, &passphrase)?;
    println!("Imported contexts from {}", file.display());
    Ok(())
}
