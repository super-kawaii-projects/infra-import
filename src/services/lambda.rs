use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("lambda");

    let fns = aws_list(&["lambda", "list-functions"], "Functions", region, profile)?;
    for f in &fns {
        let name = get_str(f, "FunctionName");
        res.add("aws_lambda_function", name, name, serde_json::json!({
            "function_name": name,
            "runtime": get_str(f, "Runtime"),
            "handler": get_str(f, "Handler"),
            "memory_size": i(f, "MemorySize"),
            "timeout": i(f, "Timeout"),
            "role": get_str(f, "Role"),
        }));
    }

    Ok(res)
}
