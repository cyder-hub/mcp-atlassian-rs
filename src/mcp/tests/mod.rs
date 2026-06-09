use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};

use crate::{
    atlassian::{auth::AtlassianAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    config::{HttpConfig, RuntimeConfig},
    confluence::{
        config::{ConfluenceConfig, ConfluenceDeployment},
        tools as confluence_tools,
    },
    context::AppContext,
    jira::config::{JiraConfig, JiraDeployment},
    jira::tools,
    tool_registry::{ToolAccess, ToolMetadata, ToolService},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use rmcp::model::{JsonObject, Tool};
use rmcp::{ServerHandler, handler::server::wrapper::Parameters};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use super::confluence_handlers::CONFLUENCE_DOWNLOAD_ATTACHMENTS_MAX_PAGES;
use super::jira_payloads::{
    parse_required_object_arg, parse_required_object_list_arg, parse_required_string_list_arg,
};
use super::*;

mod confluence_handlers;
mod discovery;
mod jira_handlers;
mod read_only;
mod request_auth;
mod schema_logging;
mod support;
