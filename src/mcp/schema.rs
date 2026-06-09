use std::sync::Arc;

use rmcp::model::Tool;
use serde_json::{Map, Value, json};

pub(super) fn sanitize_tools_for_clients(tools: Vec<Tool>) -> Vec<Tool> {
    tools.into_iter().map(sanitize_tool_for_clients).collect()
}

pub(super) fn sanitize_tool_for_clients(mut tool: Tool) -> Tool {
    let mut schema = tool.input_schema.as_ref().clone();
    sanitize_schema_object(&mut schema);
    tool.input_schema = Arc::new(schema);
    tool
}

fn sanitize_schema_object(object: &mut Map<String, Value>) {
    if object.get("default").is_some_and(Value::is_null) {
        object.remove("default");
    }

    if let Some(type_value) = object.get_mut("type") {
        sanitize_type_value(type_value);
        if type_value.as_array().is_some_and(Vec::is_empty) {
            object.remove("type");
        }
    }

    for (key, value) in object.iter_mut() {
        if key == "additionalProperties" {
            continue;
        }
        sanitize_schema_value(value);
    }
}

fn sanitize_schema_value(value: &mut Value) {
    match value {
        Value::Bool(true) => {
            *value = json!({
                "type": "object",
                "additionalProperties": true
            });
        }
        Value::Bool(false) => {
            *value = json!({ "not": {} });
        }
        Value::Array(values) => {
            for value in values {
                sanitize_schema_value(value);
            }
        }
        Value::Object(object) => {
            if object.get("default").is_some_and(Value::is_null) {
                object.remove("default");
            }

            if let Some(type_value) = object.get_mut("type") {
                sanitize_type_value(type_value);
                if type_value.as_array().is_some_and(Vec::is_empty) {
                    object.remove("type");
                }
            }

            for (key, value) in object.iter_mut() {
                if key == "additionalProperties" {
                    continue;
                }
                sanitize_schema_value(value);
            }
        }
        Value::Null | Value::Number(_) | Value::String(_) => {}
    }
}

fn sanitize_type_value(type_value: &mut Value) {
    let Value::Array(types) = type_value else {
        return;
    };

    types.retain(|value| value.as_str() != Some("null"));
    if types.len() == 1 {
        *type_value = types[0].clone();
    }
}
