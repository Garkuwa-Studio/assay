//! `assay` — CLI for Assay, by Garkuwa Labs. Placeholder; no commands yet.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "assay",
    version,
    about = "Open proof of reserves for RWAs and stablecoins on Stellar"
)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    println!("assay {}: no commands yet", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
