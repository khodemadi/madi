mod ai;
mod safety;
mod tools;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "madi", version, about = "AI-native Linux CLI")]
struct Cli {
    #[arg(required = true, trailing_var_arg = true)]
    request: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let request = cli.request.join(" ");

    let key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY is not set"))?;
    let model = std::env::var("MADI_MODEL")
        .unwrap_or_else(|_| "gpt-5.6".to_string());

    let call = ai::understand(&key, &model, &request).await?;
    safety::validate(&call)?;

    println!("\nMadi understood:\n  {}\n", call.display());

    if call.requires_confirmation() && !safety::confirm()? {
        println!("Cancelled.");
        return Ok(());
    }

    println!("{}", tools::execute(&call).await?);
    Ok(())
}

