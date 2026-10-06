use std::io;
use std::process::ExitCode;

use clap::Parser;
use zeldoc::cli::Cli;
use zeldoc::update::update_notice::UpdateNotice;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let update_notice = if cli.command.is_update() {
        None
    } else {
        UpdateNotice::start()
    };
    let code = match cli.command.run(cli.profile.as_ref()) {
        Ok(code) => code,
        // Piping into `head` closes stdout early; that is not a failure.
        Err(error) if is_broken_pipe(&error) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    };
    if let Some(update_notice) = update_notice {
        update_notice.finish(&mut io::stderr());
    }
    code
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<io::Error>())
        .any(|cause| cause.kind() == io::ErrorKind::BrokenPipe)
}
