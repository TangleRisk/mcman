use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
    core::BuildContext,
    model::Lockfile, app::App,
};

#[derive(clap::Args)]
pub struct Args {
    /// The output directory for the server
    #[arg(short, long, value_name = "file")]
    output: Option<PathBuf>,
    /// Skip some stages
    #[arg(long, value_name = "stages")]
    skip: Vec<String>,
    #[arg(long)]
    /// Don't skip downloading already downloaded jars
    force: bool,
}

pub async fn run(app: App, args: Args) -> Result<()> {
    let default_output = app.server.path.join("server");
    let output_dir = args.output.unwrap_or(default_output);

    let force = args.force;

    let skip_stages = args.skip;

    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    let mut ctx = BuildContext {
        app: &app,
        force,
        skip_stages,
        output_dir,
        lockfile: Lockfile::default(),
        new_lockfile: Lockfile::default(),
        server_process: None,
    };

    ctx.build_all().await?;

    Ok(())
}
