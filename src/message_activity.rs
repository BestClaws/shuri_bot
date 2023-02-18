use std::collections::{HashMap, VecDeque};

use crate::Bot;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use tracing::{info, error};
use serenity::{
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};

pub struct MessageActivityData;

impl TypeMapKey for MessageActivityData {
    type Value = VecDeque<(NaiveDateTime, HashMap<String, u16>)>;
}

pub async fn process_message(bot: &Bot, ctx: &Context, msg: &Message) {


    if msg.content.starts_with("!msgstats ") {
        for member in &msg.mentions {
            let user_id = member.id.to_string();
            
            let rows = sqlx::query("select * from message_activity where user_id=$1")
                .bind(user_id)
                .fetch_all(&bot.db)
                .await
                .unwrap();

            use sqlx::Row;

            let mut msg_response = String::new();

            for row in rows {
                info!("row found!");
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

    let queue = match data.get_mut::<MessageActivityData>() {
        Some(q) => q,
        None => {
            let v: VecDeque<(NaiveDateTime, HashMap<String, u16>)> = VecDeque::new();
            data.insert::<MessageActivityData>(v);
            data.get_mut::<MessageActivityData>().unwrap()
        }
    };

    info!("queue: {:?}", queue);

    // find the timeframe in data and increment/create the user message count
    // else crete the time frame
    let matched_frame = queue.iter_mut().find(|x| {
        let (timestamp, _) = x;
        timestamp.eq(&msg_timestamp)
    });

    match matched_frame {
        Some((_, hm)) => {
            let c = if let Some(c) = hm.get_mut(&user_id) {
                *c + 1
            } else {
                1
            };

            hm.insert(user_id, c);
        }
        None => {
            let mut hm = HashMap::new();
            hm.insert(user_id, 1);
            queue.push_back((msg_timestamp, hm));
        }
    }



}


#[cfg(test)]
mod testing_mod {


    struct A;

    struct B;

    trait C {
        fn get() -> Self;
    }

    impl C for A {
        fn get() -> Self {
            A
        }
    }

    impl C for B {
        fn get() -> Self {
            B
        }
    }


    #[test]
    fn test_fn() {
        let v: Vec<Box<dyn Send>> = Vec::new();


    }
}
