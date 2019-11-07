use std::collections::HashSet;

use serenity::{
    client::Context,
    framework::standard::{
        Args, CommandGroup, CommandResult, help_commands, HelpOptions, macros::help,
    },
    model::{channel::Message, id::UserId},
};

#[help]
async fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}
