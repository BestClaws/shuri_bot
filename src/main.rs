mod dictionary;
mod last_seen;
mod log_config;
mod pretty_numbers;
mod message_activity;

use std::env;

use log::{debug, error};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::event::TypingStartEvent;
use serenity::model::gateway::Ready;
use serenity::model::Timestamp;
use serenity::prelude::*;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use pretty_numbers::PrettiableNumber;




pub struct Bot {
    // TODO: add bot config here.
    pub db: Pool<Postgres>,
   
}

#[async_trait]
impl EventHandler for Bot {
    async fn typing_start(&self, _ctx: Context, typing_event: TypingStartEvent) {
        println!("typing detected");



        last_seen::record_typing_event(&self, typing_event).await;
    }


  

    // set a handler for the `message` event - so that whenever a new messagae
    // is received - the closure (or function) will be called.

    // event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        // do some event logging?
        println!(
            "message with id: {}, received at: {}",
            msg.id, msg.timestamp
        );

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
                println!("Error sending messsage: {:?}", why);
            }
        }

        // this is your poor man's modules for now.
        // until you figure out an idiomatic way
        // of doing this.
        dictionary::process_message(self, &ctx, &msg).await;
        last_seen::process_message(self, &ctx, &msg).await;
        message_activity::process_message(self, &ctx, &msg).await;
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
    // // TODO: don't hard code db name
    // let Ok(db_conn) = PgConnection::connect("postgres://postgres:yuukiwoh@localhost/shuri_bot").await else {
    //     // TODO: this is a reasonably panic. but still handle this better.
    //     panic!("critical error. cannot establish connection to database, quitting.");
    // };

    // setup logger
    use log_config::setup_loggers;

    if let Err(why) = setup_loggers() {
        println!("failed to setup logging. reason :{}", why);
    };

    debug!("hello");

    //  Initialize DB
    // TODO: dont hardcode values.
    let db_conn_pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:yuukiwoh@localhost/shuri_db")
        .await
    {
        Ok(db_conn_pool) => db_conn_pool,
        Err(e) => {
            error!(
                "critical error, failed to establish connection to database: {}",
                e
            );
            panic!();
        }
    };

    let bot = Bot { db: db_conn_pool };

    // configure the client with your discord bot token in the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    // set gateway intents, which decides what events the bot will be notified about.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_TYPING;

    // create a new instances of the client. logging in as a bot. this will
    // automaticlaly prepend your bot token with "bot" which is a  requirement
    // by discord for bot usres.
    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    // finally start a single shard and start listening to events.

    // shards will automatically attempt to reconnect and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("client error: {:?}", why);
    }
}

#[cfg(test)]
mod test_anything {

    use serenity::prelude::{TypeMap, TypeMapKey};


    #[test]
    fn test_anything() {

        // println!("{}", size_of::<Vec<u8>>());
        let mut tm = TypeMap::new();

        tm.insert::<Number>(12);
        tm.insert::<Number2>(8);

        let yes_or_no = tm.contains_key::<Number>();
        println!("{yes_or_no}");


        let yes_or_no = tm.contains_key::<Number2>();
        println!("{yes_or_no}");

        


    }

    struct Number;
    struct Number2;

    impl TypeMapKey for Number2 {
        type Value = i8;
    }


    impl TypeMapKey for Number {
        type Value = i32;
    }
}
