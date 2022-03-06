use crate::models::sandboxes::{run_python_code, run_rust_code};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
async fn rust(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let code = format!(
        "fn main() {{\nprintln!(\"{{:?}}\", {{ {} }} );\n}}",
        args.rest()
    );
    let result = run_rust_code(code).await?;
    if !result.is_empty() {
        msg.reply(ctx, result).await?;
    }
    Ok(())
}

#[command]
async fn rust_raw(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let result = run_rust_code(args.rest().to_string()).await?;
    if !result.is_empty() {
        msg.reply(ctx, result).await?;
    }
    Ok(())
}

#[command]
async fn py(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let code = format!("print({})", args.rest().trim());
    let result = run_python_code(code).await?;
    if !result.is_empty() {
        msg.reply(ctx, result).await?;
    }
    Ok(())
}

#[command]
async fn py_raw(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let result = run_python_code(args.rest().to_string()).await?;
    if !result.is_empty() {
        msg.reply(ctx, result).await?;
    }
    Ok(())
}
