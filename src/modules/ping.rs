use crate::{utils::pretty_numbers::PrettiableNumber, Bot};
use serenity::{
    model::{prelude::Message, Timestamp},
    prelude::Context,
};
use tracing::error;

pub async fn process_message(_bot: &Bot, ctx: &Context, msg: &Message) {
    if msg.content == "!ping" {
        // sending a message can fail, due to a network error, an
        // authentication error, or lack of permissions to post in the
        // channel, so log to stdout when some error happens, with a
        // description of it.

        let response_time = Timestamp::now().time() - msg.timestamp.time();
        // not using
        // let response_time = response_time.whole_microseconds().to_string().pretty();
        let response_time = response_time
            .num_microseconds()
            .unwrap()
            .to_string()
            .pretty();

        let message = format!("Pong! Took {}us", response_time);
        if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
            error!("Error sending messsage: {:?}", why);
        }
    }
}
