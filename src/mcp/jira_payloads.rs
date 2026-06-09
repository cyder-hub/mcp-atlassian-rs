use crate::{
    atlassian::error::AtlassianError,
    jira::{
        config::JiraDeployment,
        formatting::{
            comment_body_for_deployment, merge_optional_objects, parse_optional_object,
            parse_optional_string_list, parse_required_object, parse_required_object_list,
            parse_required_string_list,
        },
        tools::{
            JiraAddWorklogArgs, JiraCreateIssueArgs, JiraCreateIssueLinkArgs,
            JiraCreateRemoteIssueLinkArgs, JiraCreateSprintArgs, JiraCreateVersionArgs,
            JiraUpdateIssueArgs, JiraUpdateSprintArgs,
        },
    },
    mcp_errors::atlassian_error,
};
use rmcp::ErrorData;
use serde_json::{Map, Value, json};

use super::{optional_non_empty_arg, required_non_empty_arg};

pub(super) fn parse_optional_string_list_arg(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Vec<String>>, ErrorData> {
    parse_optional_string_list(value, field_name).map_err(atlassian_error)
}

pub(super) fn parse_required_string_list_arg(
    value: Value,
    field_name: &'static str,
) -> Result<Vec<String>, ErrorData> {
    parse_required_string_list(value, field_name).map_err(atlassian_error)
}

pub(super) fn parse_optional_object_arg(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Value>, ErrorData> {
    parse_optional_object(value, field_name).map_err(atlassian_error)
}

pub(super) fn parse_required_object_arg(
    value: Value,
    field_name: &'static str,
) -> Result<Value, ErrorData> {
    parse_required_object(value, field_name).map_err(atlassian_error)
}

pub(super) fn parse_required_object_list_arg(
    value: Value,
    field_name: &'static str,
) -> Result<Vec<Value>, ErrorData> {
    parse_required_object_list(value, field_name).map_err(atlassian_error)
}

pub(super) fn create_issue_fields_from_args(
    args: JiraCreateIssueArgs,
    deployment: JiraDeployment,
) -> Result<Value, ErrorData> {
    let project_key = required_non_empty_arg(args.project_key, "project_key")?;
    let summary = required_non_empty_arg(args.summary, "summary")?;
    let issue_type = required_non_empty_arg(args.issue_type, "issue_type")?;
    let components = parse_optional_string_list_arg(args.components, "components")?;
    let additional_fields = parse_optional_object_arg(args.additional_fields, "additional_fields")?;
    let mut fields = json!({
        "project": {"key": project_key},
        "summary": summary,
        "issuetype": {"name": issue_type},
    });

    if let Some(description) = optional_non_empty_arg(args.description) {
        fields["description"] = comment_body_for_deployment(deployment, &description);
    }
    if let Some(assignee) = optional_non_empty_arg(args.assignee) {
        let identifier_field = match deployment {
            JiraDeployment::Cloud => "accountId",
            JiraDeployment::ServerDataCenter => "name",
        };
        fields["assignee"] = json!({ identifier_field: assignee });
    }
    if let Some(components) = components {
        let components = components
            .into_iter()
            .map(|name| json!({ "name": name }))
            .collect::<Vec<_>>();
        if !components.is_empty() {
            fields["components"] = Value::Array(components);
        }
    }

    merge_optional_objects(fields, additional_fields, "additional_fields").map_err(atlassian_error)
}

pub(super) struct UpdateIssueFields {
    pub(super) issue_key: String,
    pub(super) fields: Value,
    pub(super) notify_users: Option<bool>,
}

pub(super) fn update_issue_fields_from_args(
    args: JiraUpdateIssueArgs,
    deployment: JiraDeployment,
) -> Result<(UpdateIssueFields, Option<Value>), ErrorData> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let fields = normalize_issue_fields(
        parse_required_object_arg(args.fields, "fields")?,
        deployment,
        "fields",
    )?;
    let components = parse_optional_string_list_arg(args.components, "components")?;
    let mut additional_fields =
        parse_optional_object_arg(args.additional_fields, "additional_fields")?
            .map(|value| normalize_issue_fields(value, deployment, "additional_fields"))
            .transpose()?;

    reject_unsupported_attachments(&fields, "fields")?;
    if let Some(additional_fields) = additional_fields.as_ref() {
        reject_unsupported_attachments(additional_fields, "additional_fields")?;
    }

    if let Some(components) = components {
        let components = components
            .into_iter()
            .map(|name| json!({ "name": name }))
            .collect::<Vec<_>>();
        if !components.is_empty() {
            let additional = additional_fields.get_or_insert_with(|| json!({}));
            additional["components"] = Value::Array(components);
        }
    }

    if fields.as_object().is_some_and(Map::is_empty) && additional_fields.is_none() {
        return Err(atlassian_error(AtlassianError::invalid_input(
            "fields must contain at least one update",
        )));
    }

    Ok((
        UpdateIssueFields {
            issue_key,
            fields,
            notify_users: args.notify_users,
        },
        additional_fields,
    ))
}

pub(super) fn normalize_issue_fields(
    mut fields: Value,
    deployment: JiraDeployment,
    field_name: &'static str,
) -> Result<Value, ErrorData> {
    reject_unsupported_attachments(&fields, field_name)?;
    let object = fields.as_object_mut().ok_or_else(|| {
        atlassian_error(AtlassianError::invalid_input(format!(
            "{field_name} must be a JSON object"
        )))
    })?;

    if let Some(Value::String(description)) = object.get("description").cloned() {
        object.insert(
            "description".to_string(),
            comment_body_for_deployment(deployment, &description),
        );
    }
    if let Some(Value::String(assignee)) = object.get("assignee").cloned() {
        let identifier_field = match deployment {
            JiraDeployment::Cloud => "accountId",
            JiraDeployment::ServerDataCenter => "name",
        };
        object.insert(
            "assignee".to_string(),
            json!({ identifier_field: assignee }),
        );
    }

    Ok(fields)
}

pub(super) fn reject_unsupported_attachments(
    value: &Value,
    field_name: &'static str,
) -> Result<(), ErrorData> {
    if value
        .as_object()
        .is_some_and(|object| object.contains_key("attachments"))
    {
        Err(atlassian_error(AtlassianError::invalid_input(format!(
            "{field_name}.attachments is not supported by jira_update_issue"
        ))))
    } else {
        Ok(())
    }
}

pub(super) fn version_payload_from_args(args: JiraCreateVersionArgs) -> Result<Value, ErrorData> {
    let project_key = required_non_empty_arg(args.project_key, "project_key")?;
    let name = required_non_empty_arg(args.name, "name")?;
    let mut payload = json!({
        "project": project_key,
        "name": name,
    });
    insert_optional_value(
        &mut payload,
        "startDate",
        optional_non_empty_arg(args.start_date),
    );
    insert_optional_value(
        &mut payload,
        "releaseDate",
        optional_non_empty_arg(args.release_date),
    );
    insert_optional_value(
        &mut payload,
        "description",
        optional_non_empty_arg(args.description),
    );
    Ok(payload)
}

pub(super) type WorklogPayloadParts = (String, Value, Vec<(String, String)>);

pub(super) fn add_worklog_payload_from_args(
    args: JiraAddWorklogArgs,
    deployment: JiraDeployment,
) -> Result<WorklogPayloadParts, ErrorData> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let time_spent = required_non_empty_arg(args.time_spent, "time_spent")?;
    let visibility = parse_optional_object_arg(args.visibility, "visibility")?;
    let mut payload = json!({
        "timeSpent": time_spent,
    });
    insert_optional_value(
        &mut payload,
        "started",
        optional_non_empty_arg(args.started),
    );
    if let Some(comment) = optional_non_empty_arg(args.comment) {
        payload["comment"] = comment_body_for_deployment(deployment, &comment);
    }
    if let Some(visibility) = visibility {
        payload["visibility"] = visibility;
    }

    let mut query = Vec::new();
    push_optional_query_value(&mut query, "adjustEstimate", args.adjust_estimate);
    push_optional_query_value(&mut query, "newEstimate", args.new_estimate);
    push_optional_query_value(&mut query, "reduceBy", args.reduce_by);
    Ok((issue_key, payload, query))
}

pub(super) fn issue_link_payload_from_args(
    args: JiraCreateIssueLinkArgs,
    deployment: JiraDeployment,
) -> Result<Value, ErrorData> {
    let link_type = required_non_empty_arg(args.link_type, "link_type")?;
    let inward_issue_key = required_non_empty_arg(args.inward_issue_key, "inward_issue_key")?;
    let outward_issue_key = required_non_empty_arg(args.outward_issue_key, "outward_issue_key")?;
    let mut payload = json!({
        "type": {"name": link_type},
        "inwardIssue": {"key": inward_issue_key},
        "outwardIssue": {"key": outward_issue_key},
    });

    if let Some(comment) = optional_non_empty_arg(args.comment) {
        payload["comment"] = json!({
            "body": comment_body_for_deployment(deployment, &comment)
        });
    }

    Ok(payload)
}

pub(super) fn remote_issue_link_payload_from_args(
    args: JiraCreateRemoteIssueLinkArgs,
) -> Result<(String, Value), ErrorData> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let url = required_non_empty_arg(args.url, "url")?;
    let title = required_non_empty_arg(args.title, "title")?;
    let status = parse_optional_object_arg(args.status, "status")?;
    let mut object = json!({
        "url": url,
        "title": title,
    });
    insert_optional_value(&mut object, "summary", optional_non_empty_arg(args.summary));
    if let Some(icon_url) = optional_non_empty_arg(args.icon_url) {
        object["icon"] = json!({
            "url16x16": icon_url,
            "title": object["title"].clone(),
        });
    }
    if let Some(status) = status {
        object["status"] = status;
    }

    let mut payload = json!({ "object": object });
    insert_optional_value(
        &mut payload,
        "globalId",
        optional_non_empty_arg(args.global_id),
    );
    insert_optional_value(
        &mut payload,
        "relationship",
        optional_non_empty_arg(args.relationship),
    );
    Ok((issue_key, payload))
}

pub(super) fn create_sprint_payload_from_args(
    args: JiraCreateSprintArgs,
) -> Result<Value, ErrorData> {
    let name = required_non_empty_arg(args.name, "name")?;
    let mut payload = json!({
        "name": name,
        "originBoardId": args.origin_board_id,
    });
    insert_optional_value(
        &mut payload,
        "startDate",
        optional_non_empty_arg(args.start_date),
    );
    insert_optional_value(
        &mut payload,
        "endDate",
        optional_non_empty_arg(args.end_date),
    );
    insert_optional_value(&mut payload, "goal", optional_non_empty_arg(args.goal));
    Ok(payload)
}

pub(super) fn update_sprint_payload_from_args(
    args: JiraUpdateSprintArgs,
) -> Result<(u64, Value), ErrorData> {
    let mut payload = json!({});
    insert_optional_value(&mut payload, "name", optional_non_empty_arg(args.name));
    insert_optional_value(&mut payload, "state", optional_non_empty_arg(args.state));
    insert_optional_value(
        &mut payload,
        "startDate",
        optional_non_empty_arg(args.start_date),
    );
    insert_optional_value(
        &mut payload,
        "endDate",
        optional_non_empty_arg(args.end_date),
    );
    insert_optional_value(&mut payload, "goal", optional_non_empty_arg(args.goal));

    if payload.as_object().is_some_and(Map::is_empty) {
        return Err(atlassian_error(AtlassianError::invalid_input(
            "sprint update must contain at least one field",
        )));
    }

    Ok((args.sprint_id, payload))
}

pub(super) fn version_payload_from_value(
    value: Value,
    project_key: &str,
) -> Result<Value, ErrorData> {
    let mut object = value_into_object(value, "version")?;
    let name = take_required_string_field(&mut object, "name")?;
    let start_date = take_optional_string_alias(&mut object, "startDate", "start_date")?;
    let release_date = take_optional_string_alias(&mut object, "releaseDate", "release_date")?;
    let description = take_optional_string_field(&mut object, "description")?;
    let mut payload = Value::Object(object);
    payload["project"] = Value::String(project_key.to_string());
    payload["name"] = Value::String(name);
    insert_optional_value(&mut payload, "startDate", start_date);
    insert_optional_value(&mut payload, "releaseDate", release_date);
    insert_optional_value(&mut payload, "description", description);
    Ok(payload)
}

pub(super) fn take_optional_string_alias(
    object: &mut Map<String, Value>,
    first: &'static str,
    second: &'static str,
) -> Result<Option<String>, ErrorData> {
    match take_optional_string_field(object, first)? {
        Some(value) => Ok(Some(value)),
        None => take_optional_string_field(object, second),
    }
}

pub(super) fn insert_optional_value(payload: &mut Value, key: &'static str, value: Option<String>) {
    if let Some(value) = value {
        payload[key] = Value::String(value);
    }
}

pub(super) fn push_optional_query_value(
    query: &mut Vec<(String, String)>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = optional_non_empty_arg(value) {
        query.push((key.to_string(), value));
    }
}

pub(super) fn batch_create_issue_updates_from_args(
    issues: Value,
    deployment: JiraDeployment,
) -> Result<Vec<Value>, ErrorData> {
    parse_required_object_list_arg(issues, "issues")?
        .into_iter()
        .map(|issue| {
            create_issue_fields_from_value(issue, deployment)
                .map(|fields| json!({ "fields": fields }))
        })
        .collect()
}

pub(super) fn create_issue_fields_from_value(
    issue: Value,
    deployment: JiraDeployment,
) -> Result<Value, ErrorData> {
    let mut fields = value_into_object(issue, "issue")?;
    let project_key = take_required_string_field(&mut fields, "project_key")?;
    let summary = take_required_string_field(&mut fields, "summary")?;
    let issue_type = take_required_string_field(&mut fields, "issue_type")?;
    let assignee = take_optional_string_field(&mut fields, "assignee")?;
    let description = take_optional_string_field(&mut fields, "description")?;
    let components = fields.remove("components");
    let additional_fields = if fields.is_empty() {
        None
    } else {
        Some(Value::Object(fields))
    };

    create_issue_fields_from_args(
        JiraCreateIssueArgs {
            project_key,
            summary,
            issue_type,
            assignee,
            description,
            components,
            additional_fields,
        },
        deployment,
    )
}

pub(super) fn value_into_object(
    value: Value,
    field_name: &'static str,
) -> Result<Map<String, Value>, ErrorData> {
    match parse_required_object_arg(value, field_name)? {
        Value::Object(object) => Ok(object),
        _ => unreachable!("parse_required_object_arg only returns JSON objects"),
    }
}

pub(super) fn take_required_string_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<String, ErrorData> {
    match object.remove(field_name) {
        Some(Value::String(value)) => required_non_empty_arg(value, field_name),
        Some(_) => Err(atlassian_error(AtlassianError::invalid_input(format!(
            "{field_name} must be a string"
        )))),
        None => Err(atlassian_error(AtlassianError::invalid_input(format!(
            "{field_name} is required"
        )))),
    }
}

pub(super) fn take_optional_string_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<Option<String>, ErrorData> {
    match object.remove(field_name) {
        Some(Value::String(value)) => Ok(optional_non_empty_arg(Some(value))),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(atlassian_error(AtlassianError::invalid_input(format!(
            "{field_name} must be a string"
        )))),
    }
}
