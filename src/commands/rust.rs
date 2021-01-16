use code_sandbox::{Engine, ExecuteRequest, Sandbox};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

async fn run_code(code: String, ctx: &mut Context, msg: &Message) -> CommandResult {
    let sandbox = Sandbox::new()?;
    let req = ExecuteRequest {
        code,
        engine: Engine::Rust,
    };

    let response = sandbox.execute(&req).await?;

    let output = response.stdout.trim();
    if output.is_empty() {
        let err_output = response.stderr.trim();
        if !err_output.is_empty() {
            msg.reply(ctx, err_output).await?;
        }
    } else {
        msg.reply(ctx, output).await?;
    }

    Ok(())
}

#[command]
async fn rust(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let code = format!(
        "fn main() {{\nprintln!(\"{{:?}}\", {{ {} }} );\n}}",
        args.rest()
    );
    run_code(code, ctx, msg).await
}

#[command]
async fn rust_raw(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let code = args.rest().to_string();
    run_code(code, ctx, msg).await
}
