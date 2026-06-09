use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    atlassian::auth::AtlassianAuth,
    jira::config::{DEFAULT_JIRA_TIMEOUT_SECONDS, JiraDeployment},
};

use super::*;

mod comments_transitions;
mod errors;
mod extensions;
mod fields;
mod issues;
mod metrics;
mod search;
mod support;
