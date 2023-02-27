use chrono::NaiveDateTime;
// use chrono::{NaiveDateTime, Utc};
use serenity::model::prelude::{ActivityType, Presence, UserId};
use serenity::prelude::*;
use sqlx::{Pool, Postgres};
// use sqlx::Row;
use tracing::{error, info};

use crate::Bot;

#[tracing::instrument(
    skip(bot, ctx, data),
    fields(
        user =  %data.user.id.to_string()
    )
)]
pub async fn presence_update(bot: &Bot, ctx: Context, data: &Presence) {

    let Ok(user) = ctx.http.get_user(data.user.id.into()).await else {
        return;
    };

    info!("processing for: {:#?}",user);
    // skip if bot.

    if user.bot {
        info!(
            "found presence status for {}, but skipped cuz bot",
            user.id
        );
        return;
    }

    info!(
        "found presence status for {}",
        user.id
    );



    

    for activity in &data.activities {
        if activity.kind == ActivityType::Playing {
            let user_id = &data.user.id;
            let game_name = &activity.name;
            let launched_at = &activity
                .timestamps
                .as_ref()
                .and_then(|t| NaiveDateTime::from_timestamp_opt(t.start.unwrap_or(0) as i64, 0));

            let pgr = PlayedGameRecord {
                user_id,
                game_name,
                launched_at,
            };

            pgr.save(&bot.db).await;
        }
    }
}

// TODO: this is a data model object are the lifetimes really necessary?
// Why not just clone it.
#[derive(Debug)]
pub struct PlayedGameRecord<'a> {
    user_id: &'a UserId,
    game_name: &'a String,
    launched_at: &'a Option<NaiveDateTime>,
}

impl<'a> PlayedGameRecord<'a> {
    pub async fn save(&self, pool: &Pool<Postgres>) {
        // do something with this result
        // if the result is an error how do you handle this?
        // give reasoning
        let query_result = sqlx::query(
            "INSERT INTO played_game_records (user_id, game_name, launched_at) VALUES ($1, $2, $3)",
        )
        .bind(self.user_id.to_string())
        .bind(&self.game_name)
        .bind(self.launched_at)
        .execute(pool)
        .await;

        match query_result {
            Ok(query_result) => {
                if query_result.rows_affected() == 1 {
                    info!("succesfully added PlayedGameRecord: {:#?}", self);
                }
            }

            Err(e) => {
                // failing to record a game record isn't the end of the world
                // so just do nothing
                error!("failed to save the game record to db. ERR: {:#?}", e);
            }
        }
    }
}
