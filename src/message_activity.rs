use std::collections::{HashMap, VecDeque};

use crate::Bot;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use log::info;
use serenity::{
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};

struct MessageActivityData;

impl TypeMapKey for MessageActivityData {
    type Value = VecDeque<(NaiveDateTime, HashMap<String, u16>)>;
}

pub async fn process_message(bot: &Bot, ctx: &Context, msg: &Message) {
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

    println!("here it is: {} {queue:?}", queue.len());

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



    // push to db every 10 seconds.
    // TODO: improve this.
    // TODO: if database stuff fails push_front the failed timestamp records.
    // you need to learn rollback commit for this. (for now let's just assume
    // db doesn't fail on you.)

    
    let modu = Utc::now().timestamp() % 10;
    info!("timestamp: {}, modulus: {modu}", msg_timestamp.timestamp());
    if modu == 0 {
        info!("persisting to message_activity");
        while queue.len() > 1 { // leave atleast one latest timestap alone incase
                                // its still being populated.

            info!("more than 2 entries found in message_activity queue");
            let Some(e) = queue.pop_front() else {
                panic!("error occured getting the next element.");
            };

            let timestamp = e.0.timestamp();
            
            for (user_id, message_count) in &e.1 {
                let res = sqlx::query("INSERT INTO message_activiy (user_id, timestamp, count) values($1, $2, $3)")
                    .bind(user_id)
                    .bind(timestamp)
                    .bind(*message_count as i32)
                    .execute(&bot.db).await;
            }
        }     
    }


}
