use chrono::NaiveDate;
// use chrono::{NaiveDateTime, Utc};
use serenity::model::prelude::{Presence, ActivityType, UserId};
use serenity::model::user::User;
use serenity::prelude::*;
use sqlx::{Postgres, Pool};
// use sqlx::Row;
use tracing::{info};

use crate::Bot;

pub async fn presence_update(_bot: &Bot, _ctx: Context, data: &Presence) {

    // skip if bot.
    if let Some(bot) = data.user.bot {
        if bot { return; }
    };

    
    
    for activity in &data.activities {
        if activity.kind == ActivityType::Playing {
            let user_id = &data.user.id;
            let game_name = &activity.name;
            let timestamps = &activity.timestamps;

            info!("id: {user_id},  game name: {game_name}, timestamps: {timestamps:?}");

        }
    }
}


pub struct PlayedGameRecord {
    user_id: UserId,
    game_name: String,
    launched_at: NaiveDate,
}

impl PlayedGameRecord {
    
    pub async fn save(&self, pool: &Pool<Postgres>) {
        let result = sqlx::query("INSERT INTO played_game_record (user_id, game_name, launched_at) VALUES ($1, $2, $3)")
            .bind(self.user_id.to_string())
            .bind(&self.game_name)
            .bind(self.launched_at)
            .execute(pool).await;

    }
}

