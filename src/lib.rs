#![feature(fmt_internals)]
#![feature(test)]

pub(crate) mod bot;
pub(crate) mod cron;
pub(crate) mod database;
pub(crate) mod http;
pub(crate) mod ratelimit;
pub(crate) mod util;

pub use bot::start_bot;
pub use cron::start_cron;
pub use http::start_actix;
pub use prelude::{CLIENT, START_TIME};

pub(crate) mod prelude {
    lazy_static! {
        pub static ref CLIENT: reqwest::Client = reqwest::Client::new();
        pub static ref START_TIME: OnceCell<Instant> = OnceCell::new();
        pub static ref HTTP: OnceCell<Arc<Client>> = OnceCell::new();
        pub static ref REVIEWS: OnceCell<Vec<Comment>> = OnceCell::new();
    }
    pub use super::Lang;
    pub use crate::{
        bot::{bot::Comment, utils::*},
        database::{queries::*, structs::*, *},
        util::*,
    };
    pub use actix_multipart::Multipart;
    pub use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
    pub use anyhow::{anyhow, Result};
    pub use byte_unit::Byte;
    pub use cached::{proc_macro::cached, Cached, CachedAsync};
    pub use futures::{future, StreamExt, TryStreamExt};
    pub use humantime::format_duration;
    pub use lazy_static::lazy_static;
    pub use once_cell::sync::OnceCell;
    pub use paperclip::actix::*;
    pub use rand::{prelude::SliceRandom, Rng};
    pub use rosetta_i18n::Language;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{from_str, json, Value};
    pub use std::{
        collections::HashMap,
        env, fs,
        fs::{metadata, read},
        io,
        io::{Cursor, Write},
        path::PathBuf,
        process::{self, Command as ProcessCommand},
        slice::Iter,
        sync::Arc,
        time::Instant,
    };
    pub use tokio::fs::create_dir_all;
    pub use tokio_pg_mapper::FromTokioPostgresRow;
    pub use ts_rs::TS;
    pub use twilight_gateway::{cluster::ShardScheme, Cluster, Event, Intents};
    pub use twilight_http::Client;
    pub use twilight_model::{
        application::{
            command::{Command, CommandType},
            interaction::{
                application_command::{CommandDataOption, CommandOptionValue},
                ApplicationCommand, Interaction,
            },
        },
        channel::{
            embed::EmbedFooter,
            message::{AllowedMentions, MessageFlags},
        },
        id::*,
    };
    pub use twilight_util::builder::command::*;
}

rosetta_i18n::include_translations!();
