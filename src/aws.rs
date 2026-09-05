use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::process::Command;

/// Run an AWS CLI command and return parsed JSON output.
/// `args` should be like ["ec2", "describe-vpcs"]
/// Region and profile are injected automatically.
pub fn aws_cli(service_args: &[&str], region: &str, profile: Option<&str>) -> Result<Value> {
    let mut cmd = Command::new("aws");
    cmd.args(service_args);
    cmd.args(["--output", "json", "--region", region]);

    if let Some(p) = profile {
        cmd.args(["--profile", p]);
    }

    let output = cmd.output()
        .context("Failed to execute AWS CLI. Is 'aws' installed and in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("AWS CLI error: {}", stderr.trim());
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse AWS CLI JSON output")?;

    Ok(json)
}

/// Run AWS CLI and return the array found at `key` in the response.
/// Returns empty vec if key is missing or null.
pub fn aws_list(service_args: &[&str], key: &str, region: &str, profile: Option<&str>) -> Result<Vec<Value>> {
    let json = aws_cli(service_args, region, profile)?;
    Ok(json.get(key)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default())
}

/// Extract a string field from a JSON value, defaulting to ""
pub fn s(val: &Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Extract a string field as &str reference with lifetime tied to the Value
pub fn get_str<'a>(val: &'a Value, key: &str) -> &'a str {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
}

/// Extract an i64 field, defaulting to 0
pub fn i(val: &Value, key: &str) -> i64 {
    val.get(key)
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// Extract a bool field, defaulting to false
pub fn b(val: &Value, key: &str) -> bool {
    val.get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Get the "Name" tag from a Tags array, or fallback to ""
pub fn get_name(val: &Value) -> String {
    val.get("Tags")
        .and_then(|t| t.as_array())
        .and_then(|tags| {
            tags.iter().find(|t| get_str(t, "Key") == "Name")
        })
        .map(|t| s(t, "Value"))
        .unwrap_or_default()
}

/// Extract tags as a JSON object (excluding aws: prefixed tags)
pub fn extract_tags(val: &Value) -> Value {
    let tags = val.get("Tags")
        .and_then(|t| t.as_array());

    match tags {
        Some(arr) => {
            let map: serde_json::Map<String, Value> = arr.iter()
                .filter(|t| !get_str(t, "Key").starts_with("aws:"))
                .map(|t| (s(t, "Key"), Value::String(s(t, "Value"))))
                .collect();
            Value::Object(map)
        }
        None => Value::Object(serde_json::Map::new()),
    }
}

/// Verify AWS CLI is installed and credentials work. Returns (account, arn).
pub fn verify_identity(region: &str, profile: Option<&str>) -> Result<(String, String)> {
    let json = aws_cli(&["sts", "get-caller-identity"], region, profile)?;
    let account = s(&json, "Account");
    let arn = s(&json, "Arn");
    Ok((account, arn))
}
