use city_common::cli::args::APIServerArgs;

use crate::build;

#[tokio::main]
pub async fn run(_args: APIServerArgs) -> anyhow::Result<()> {
    println!(
        "
----------------------------------------
|           CityRollup v{}          |
----------------------------------------
",
        build::PKG_VERSION
    );
    Ok(())
}
