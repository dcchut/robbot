use code_sandbox::SandboxBuilder;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

async fn run_code(code: String, ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut builder = SandboxBuilder::new(
        "dcchut/code-sandbox-rust-stable",
        vec!["cargo", "run", "--release"],
    )?;

    builder.mount("/playground/src/main.rs", code)?;

    let sandbox = builder.build()?;
    let result = sandbox.execute().await?;

    let mut reply = result.stdout().trim_end();

    if reply.is_empty() {
        reply = result.stderr().trim_end();
    }

    // Only send a reply if we have something to say
    if !reply.is_empty() {
        msg.reply(ctx, reply).await?;
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
