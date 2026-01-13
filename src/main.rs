use anyhow::Result;
use langdb::repl;

fn main() -> Result<()> {
    if let Err(e) = repl::run_repl() {
         eprintln!("Fatal error: {}", e);
         std::process::exit(1);
    }
    Ok(())
}
