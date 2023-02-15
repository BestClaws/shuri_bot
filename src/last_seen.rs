use chrono::{NaiveDateTime, Utc};
use serenity::model::{channel::Message, event::TypingStartEvent};
use serenity::prelude::*;
use sqlx::Row;

use crate::Bot;

pub async fn record_typing_event(bot: &Bot, event: TypingStartEvent) {
    let t = NaiveDateTime::from_timestamp_opt(event.timestamp as i64, 0);

    let res = sqlx::query("insert into last_seen (user_id, last_seen) values($1, $2) on conflict (user_id) do update  set last_seen = $2")
        .bind(event.user_id.to_string())
        .bind(t)
        .execute(&bot.db)
        .await
        .unwrap();

    println!("{res:?}");
}

pub async fn process_message(bot: &Bot, ctx: &Context, msg: &Message) {
    if msg.content.starts_with("!seen ") {
        for member in &msg.mentions {
            // TODO: learn what exactly for in does or calls. and how references and owned values behave.

            let x = sqlx::query("SELECT last_seen FROM last_seen WHERE user_id = $1")
                .bind(member.id.to_string())
                .fetch_one(&bot.db)
                .await
                .unwrap();

            let t: chrono::DateTime<Utc> = x.try_get("last_seen").unwrap();

            let msg_response = format!("last seen: <t:{}:f>", t.timestamp());

            if let Err(why) = msg.channel_id.say(&ctx.http, msg_response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}
