use crate::migrate;
use crate::opt::ConnectOpts;
use console::style;
use promptly::{prompt, ReadlineError};
use sqlx::any::Any;
use sqlx::migrate::MigrateDatabase;

pub async fn create(connect_opts: &ConnectOpts) -> anyhow::Result<()> {
    // NOTE: only retry the idempotent action.
    // We're assuming that if this succeeds, then any following operations should also succeed.
    let exists = crate::retry_connect_errors(connect_opts, Any::database_exists).await?;

    if !exists {
        Any::create_database(&connect_opts.database_url).await?;
    }

    Ok(())
}

pub async fn drop(connect_opts: &ConnectOpts, confirm: bool) -> anyhow::Result<()> {
    if confirm && !ask_to_continue(connect_opts) {
        return Ok(());
    }

    // NOTE: only retry the idempotent action.
    // We're assuming that if this succeeds, then any following operations should also succeed.
    let exists = crate::retry_connect_errors(connect_opts, Any::database_exists).await?;

    if exists {
        Any::drop_database(&connect_opts.database_url).await?;
    }

    Ok(())
}

pub async fn reset(
    migration_source: &str,
    connect_opts: &ConnectOpts,
    confirm: bool,
) -> anyhow::Result<()> {
    drop(connect_opts, confirm).await?;
    setup(migration_source, connect_opts).await
}

pub async fn setup(migration_source: &str, connect_opts: &ConnectOpts) -> anyhow::Result<()> {
    create(connect_opts).await?;
    migrate::run(migration_source, connect_opts, false, false).await
}

fn ask_to_continue(connect_opts: &ConnectOpts) -> bool {
    loop {
        let r: Result<String, ReadlineError> = prompt(format!(
            "Drop database at {}? (y/n)",
            style(&connect_opts.database_url).cyan()
        ));
        match r {
            Ok(response) => {
                if response == "n" || response == "N" {
                    return false;
                } else if response == "y" || response == "Y" {
                    return true;
                } else {
                    println!(
                        "Response not recognized: {}\nPlease type 'y' or 'n' and press enter.",
                        response
                    );
                }
            }
            Err(e) => {
                println!("{}", e);
                return false;
            }
        }
    }
}
