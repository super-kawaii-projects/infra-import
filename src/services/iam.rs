use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("iam");

    // Roles
    let roles = aws_list(&["iam", "list-roles"], "Roles", region, profile)?;
    for r in &roles {
        let path = get_str(r, "Path");
        if path.starts_with("/aws-service-role/") { continue; }
        let name = get_str(r, "RoleName");
        res.add("aws_iam_role", name, name, serde_json::json!({
            "name": name,
            "arn": get_str(r, "Arn"),
            "path": path,
            "assume_role_policy": get_str(r, "AssumeRolePolicyDocument"),
        }));
    }

    // Policies (customer-managed only, Scope = Local)
    let policies = aws_list(&["iam", "list-policies", "--scope", "Local"], "Policies", region, profile)?;
    for p in &policies {
        res.add("aws_iam_policy", get_str(p, "Arn"), get_str(p, "PolicyName"), serde_json::json!({
            "name": get_str(p, "PolicyName"),
            "arn": get_str(p, "Arn"),
            "path": get_str(p, "Path"),
        }));
    }

    // Users
    let users = aws_list(&["iam", "list-users"], "Users", region, profile)?;
    for u in &users {
        let name = get_str(u, "UserName");
        res.add("aws_iam_user", name, name, serde_json::json!({
            "name": name,
            "path": get_str(u, "Path"),
        }));
    }

    // Groups
    let groups = aws_list(&["iam", "list-groups"], "Groups", region, profile)?;
    for g in &groups {
        let name = get_str(g, "GroupName");
        res.add("aws_iam_group", name, name, serde_json::json!({
            "name": name,
            "path": get_str(g, "Path"),
        }));
    }

    // Instance Profiles
    let profiles = aws_list(&["iam", "list-instance-profiles"], "InstanceProfiles", region, profile)?;
    for ip in &profiles {
        let name = get_str(ip, "InstanceProfileName");
        let role_names: Vec<String> = ip.get("Roles")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|r| s(r, "RoleName")).collect())
            .unwrap_or_default();
        res.add("aws_iam_instance_profile", name, name, serde_json::json!({
            "name": name,
            "roles": role_names,
        }));
    }

    Ok(res)
}
