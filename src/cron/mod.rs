use std::time::Duration;

use futures::stream;
use queries::get_images::delete_all;
use tokio::time;

use crate::prelude::*;

pub async fn start_cron() {
    let mut interval = time::interval(Duration::from_secs(86400000));

    tokio::spawn(async move {
        loop {
            tokio::spawn(async move {
                if let Some(client) = HTTP.get() {
                    let users = get_users_autodelete::exec().await;
                    if users.is_err() {
                        return;
                    }
                    let mut summary = Vec::new();
                    for user in users.unwrap() {
                        let amount = delete_all(user.0, user.1).await;
                        if let Ok(amount) = amount {
                            if amount != 0 {
                                summary.push(format!("{}: `{}`", user.2, amount));
                            }
                        }
                    }
                    if summary.is_empty() {
                        return;
                    }

                    let embed = create_embed()
                        .title("Deleted images summary")
                        .description(&summary.join("\n"))
                        .build();
                    client
                        .create_message(Id::new(929698255300882522u64))
                        .embeds(&vec![embed])
                        .unwrap()
                        .exec()
                        .await
                        .ok();
                }
            });
            interval.tick().await;
        }
    });
}
