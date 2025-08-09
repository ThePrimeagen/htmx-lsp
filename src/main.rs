mod opts;

use std::{fs::File, io::stderr};

use anyhow::Result;
use clap::Parser;
use log::{error, trace};
use structured_logger::{json::new_writer, Builder};

use opts::HtmxLspConfig;

use htmx_lsp_server::start_lsp;

fn main() -> Result<()> {
    let config = HtmxLspConfig::parse();

    let mut builder = Builder::with_level(&config.level);

    if let Some(file) = &config.file {
        let log_file = match File::options().create(true).append(true).open(file) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to open log file: \"{file}\" -- {e}");
                std::process::exit(1);
            }
        };

        builder = builder.with_target_writer("*", new_writer(log_file));
    } else {
        builder = builder.with_target_writer("*", new_writer(stderr()))
    }

    builder.init();
    trace!("log options: {:?}", config);

    start_lsp()?;

    Ok(())
}
