use std::{
    collections::{BTreeSet, VecDeque},
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    atlassian::{auth::AtlassianAuth, error::AtlassianError},
    confluence::config::{ConfluenceConfig, ConfluenceDeployment},
};

use super::*;

mod analytics;
mod attachments;
mod comments;
mod core;
mod labels;
mod pages;
mod search;
mod support;
mod users;
