mod modules;
mod utils;

use std::env;

use tracing::{debug, error, info};

use serenity::model::channel::Message;
use serenity::model::event::TypingStartEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::Presence};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};


use modules::*;
use message_activity::MessageActivityData;


#[derive(Debug)]
pub struct Bot {
    // TODO: add bot config here.
    pub db: Pool<Postgres>,
}

#[async_trait]
impl EventHandler for Bot {
    async fn presence_update(&self, ctx: Context, new_data: Presence) {
        most_played_game::presence_update(self, ctx, &new_data).await;
    }

    async fn typing_start(&self, _ctx: Context, typing_event: TypingStartEvent) {
        debug!("received typing event...");
        last_seen::record_typing_event(&self, typing_event).await;
    }

    // set a handler for the `message` event - so that whenever a new messagae
    // is received - the closure (or function) will be called.

    // event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        info!(
            "message with id: {}, received at: {}",
            msg.id, msg.timestamp
        );


        // this is your poor man's modules for now.
        // until you figure out an idiomatic way
        // of doing this.
        ping::process_message(self, &ctx, &msg).await;
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
        info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {

    // TODO: do this in another file.
    // setup logging , or rather tracing.

    // a blocking file appender
    let file_appender = tracing_appender::rolling::never("logs", "output.log");
    
    // make it non blocking
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // use the non blocking one to write events.
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    // if you intend to use tokio console. use this instead. 
    // console_subscriber::init();


    //  Initialize DB
    // TODO: dont hardcode values.
    let postgres_url = env::var("POSTGRES_URL").expect("Expected a postgres url in the environment");
    let db_conn_pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await
    {
        Ok(db_conn_pool) =>  {
            error!("connection to db succesful");
            db_conn_pool
        }
        Err(err) => {
            error!(
                "critical error, failed to establish connection to database: {}",
                err
            );
            panic!("{}", err);
        }
    };

    let bot = Bot { db: db_conn_pool };

    // configure the client with your discord bot token in the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    // set gateway intents, which decides what events the bot will be notified about.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_TYPING
        | GatewayIntents::GUILD_PRESENCES;

    // grab a clone of pool so it can be sent to tokio task
    let pool = bot.db.clone();

    // create a new instances of the client. logging in as a bot. this will
    // automaticlaly prepend your bot token with "bot" which is a  requirement
    // by discord for bot usres.
    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    // do module wise setup
    

    // persist message activity to db every n seconds.
    let data = client.data.clone();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
    tokio::spawn(async move {
        loop {
            interval.tick().await;
            let mut data = data.write().await;
            let data = data.get_mut::<MessageActivityData>();
            if let Some(msg_activity_queue) = data {
                debug!("persisting to message_activity");
                while msg_activity_queue.len() > 1 {
                    // leave atleast one latest timestap alone incase
                    // its still being populated.

                    info!("more than one entry found in message_activity queue");
                    let Some(e) = msg_activity_queue.pop_front() else {
                        panic!("error occured getting the next element.");
                    };

                    let timestamp = e.0;

                    for (user_id, message_count) in &e.1 {
                        let res = sqlx::query("INSERT INTO message_activity (user_id, timestamp, message_count) values($1, $2, $3)")
                            .bind(user_id)
                            .bind(timestamp)
                            .bind(*message_count as i32)
                            .execute(&pool).await;

                        info!("insert results: {:?}", res)
                    }
                }
            }
        }
    });


    // start a single shard and start listening to events.
    // shards will automatically attempt to reconnect and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        error!("client error: {:?}", why);
    }
}
