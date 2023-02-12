use chrono::{Utc, TimeZone};
use serenity::prelude::*;
use serenity::{model::{channel::Message, event::TypingStartEvent}};
use sqlx::Row;

use crate::Bot;

pub async fn record_typing_event(bot: &Bot, event: TypingStartEvent) {
    // typing_event.timestamp is in seconds not millis so convert into millis first.
    let t = Utc
        .timestamp_millis_opt(event.timestamp as i64 * 1000)
        .unwrap();

    println!("{}", event.timestamp);

    // let t = NaiveDateTime::from_timestamp_millis(typing_event.timestamp as i64).unwrap(); // TODO: u64 time to i64 hm..anything u can do?

    let res = sqlx::query("insert into last_seen (user_id, last_seen) values($1, $2) on conflict (user_id) do update  set last_seen = $2")
        .bind(event.user_id.to_string())
        .bind(t)
        .execute(&bot.db)
        .await
        .unwrap();

    println!("{res:?}");
}


pub async fn process_message(bot: &Bot, ctx: Context, msg: Message) {
    if msg.content.starts_with("!seen ") {
        for member in msg.mentions {
  

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


pub async fn fetch_last_seen() {

}