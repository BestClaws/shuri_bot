use std::collections::{HashMap, VecDeque};

use crate::Bot;
use chrono::{DateTime, NaiveDateTime, Utc};
use serenity::{
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};
use tracing::{error, info};

// serenity data model to store message activity.
pub struct MessageActivityData;

impl TypeMapKey for MessageActivityData {
    type Value = VecDeque<(NaiveDateTime, HashMap<String, u16>)>;
}

pub async fn process_message(bot: &Bot, ctx: &Context, msg: &Message) {
    if msg.content.starts_with("!msgstats ") {
        info!("retrieving message stats for {}", msg.author.id);
        for member in &msg.mentions {
            let user_id = member.id.to_string();

            let rows = sqlx::query("select * from message_activity where user_id=$1")
                .bind(user_id)
                .fetch_all(&bot.db)
                .await
                .unwrap();

            use sqlx::Row;

            let mut msg_response = String::new();

            info!("found {} message points", rows.len());

            for row in rows {
                let t: DateTime<Utc> = row.try_get("timestamp").unwrap();
                let c: i32 = row.try_get("message_count").unwrap();

                let r = format!("time: {t}, count: {c}\n");
                msg_response.push_str(&r);
            }

            if let Err(why) = msg.channel_id.say(&ctx.http, msg_response).await {
                error!("error sending message: {:?}", why);
            }
        }
    }

    update_activity_queue(ctx, msg).await;

}


async fn update_activity_queue(ctx: &Context, msg: &Message) {

    // (cache should be a queue)
    // check message timestamp (ignoring second) in cache
    // if present increase count, or add entry
    // AND
    // if queue size is greater than 1 (meaning old timestmap is present.)
    // take it out and persist data to db
    // queue should have feature untake in case db fails
    let mut data = ctx.data.write().await;

    // strip the ms and ns from datetime.
    let msg_timestamp = NaiveDateTime::from_timestamp_opt(msg.timestamp.timestamp(), 0).unwrap();
    let user_id = msg.author.id.to_string();

 
    // TODO: this is a one time thing. shouldnt exist here.
    let msg_activity_queue = match data.get_mut::<MessageActivityData>() {
        Some(q) => q,
        None => {
            let v: VecDeque<(NaiveDateTime, HashMap<String, u16>)> = VecDeque::new();
            data.insert::<MessageActivityData>(v);
            data.get_mut::<MessageActivityData>().unwrap()
        }
    };


    info!("queue: {:?}", msg_activity_queue);

    // find the timeframe in data and increment/create the user message count
    // else crete the time frame
    let time_frame = msg_activity_queue.iter_mut().find(|x| {
        let (timestamp, _) = x;
        timestamp.eq(&msg_timestamp)
    });

    match time_frame {
        Some((_, user_msg_counts)) => {
            let count = if let Some(count) = user_msg_counts.get_mut(&user_id) {
                *count + 1
            } else {
                1
            };

            user_msg_counts.insert(user_id, count);
        }
        None => {
            let mut user_msg_counts = HashMap::new();
            user_msg_counts.insert(user_id, 1);
            msg_activity_queue.push_back((msg_timestamp, user_msg_counts));
        }
    }
}