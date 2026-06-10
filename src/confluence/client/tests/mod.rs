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
    confluence::config::{ConfluenceConfig, ConfluenceDeployment},
    upstream::{auth::UpstreamAuth, error::UpstreamError},
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
