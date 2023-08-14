mod opts;

use std::{fs::File, io::stderr};

use anyhow::Result;
use clap::Parser;
use log::trace;
use structured_logger::{json::new_writer, Builder};

use opts::JSPerfLspConfig;

use lsp::start_lsp;

fn main() -> Result<()> {
    let config = JSPerfLspConfig::parse();

    let mut builder = Builder::with_level(&config.level);

    if let Some(file) = &config.file {
        let log_file = File::options()
            .create(true)
            .append(true)
            .open(file)
            .unwrap();

        builder = builder.with_target_writer("*", new_writer(log_file));
    } else {
        builder = builder.with_target_writer("*", new_writer(stderr()))
    }

    builder.init();
    trace!("log options: {:?}", config);

    start_lsp()?;

    return Ok(());
}
