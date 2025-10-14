use anyhow::Result;
use jsonrpsee::{Extensions, RpcModule, server::ServerBuilder, types::ErrorObjectOwned};
use minichain::run;

#[tokio::main]
async fn main() -> Result<()> {
    run();

    let ctx = ();

    let server = ServerBuilder::default().build("127.0.0.1:8080").await?;

    let mut module: RpcModule<()> = RpcModule::new(ctx);

    module.register_method(
        "add",
        |params, _ctx: &(), _ext: &Extensions| -> Result<u64, ErrorObjectOwned> {
            let (a, b): (u64, u64) = params.parse()?;
            Ok(a + b)
        },
    )?;

    let handle = server.start(module);
    println!("JSON-RPC on http://127.0.0.1:8080");
    handle.stopped().await;
    Ok(())

    
}
