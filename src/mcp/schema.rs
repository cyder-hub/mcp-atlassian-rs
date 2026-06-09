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
    if let Some(output_schema) = tool.output_schema.as_ref() {
        let mut schema = output_schema.as_ref().clone();
        sanitize_schema_object(&mut schema);
        tool.output_schema = Some(Arc::new(schema));
    }
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
        match key.as_str() {
            "properties" | "patternProperties" | "$defs" | "definitions" | "dependentSchemas" => {
                sanitize_schema_map(value)
            }
            "anyOf" | "oneOf" | "allOf" | "prefixItems" => sanitize_schema_array(value),
            "items"
            | "additionalItems"
            | "contains"
            | "not"
            | "if"
            | "then"
            | "else"
            | "propertyNames"
            | "unevaluatedItems"
            | "unevaluatedProperties" => {
                sanitize_schema_value(value);
            }
            _ => {}
        }
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
        Value::Object(object) => sanitize_schema_object(object),
        Value::Null | Value::Number(_) | Value::String(_) => {}
    }
}

fn sanitize_schema_map(value: &mut Value) {
    let Value::Object(schemas) = value else {
        return;
    };

    for schema in schemas.values_mut() {
        sanitize_schema_value(schema);
    }
}

fn sanitize_schema_array(value: &mut Value) {
    let Value::Array(schemas) = value else {
        return;
    };

    for schema in schemas {
        sanitize_schema_value(schema);
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
