mod pretty_numbers;

use std::env;

use serenity::async_trait;
use serenity::model::Timestamp;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::pretty_numbers::PrettiableNumber;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // set a handler for the `message` event - so that whenever a new messagae
    // is received - the closure (or function) will be called.

    // event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {

        // do some event logging?
        println!("message with id: {}, received at: {}",
         msg.id, msg.timestamp);

        if msg.content == "!ping" {
            // sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.

            let response_time = Timestamp::now().time() - msg.timestamp.time();
            let response_time = response_time.whole_microseconds().to_string().pretty();

            let message = format!("Pong! Took {}us", response_time);
            if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                println!("Error sending messsage: {:?}", why);
            }
        }   
    }

    // set a handler to be called on the `ready` event. This is called when
    // a shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.

    // In this case, just print what the current user's username is
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // configure the client with your discord bot token in the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    // set gateway intents, which decides what events the bot will be notified about.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // create a new instances of the client. logging in as a bot. this will
    // automaticlaly prepend your bot token with "bot" which is a  requirement
    // by discord for bot usres.
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // finally start a single shard and start listening to events.

    // shards will automatically attempt to reconnect and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("client error: {:?}", why);
    }
}
