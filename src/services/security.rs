use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>, vpc_filter: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("security");

    // Security Groups
    let sgs = aws_list(&["ec2", "describe-security-groups"], "SecurityGroups", region, profile)?;
    for sg in &sgs {
        if get_str(sg, "GroupName") == "default" { continue; }
        if let Some(f) = vpc_filter {
            if get_str(sg, "VpcId") != f { continue; }
        }
        let ingress_count = sg.get("IpPermissions").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let egress_count = sg.get("IpPermissionsEgress").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        res.add("aws_security_group", get_str(sg, "GroupId"), get_str(sg, "GroupName"), serde_json::json!({
            "name": get_str(sg, "GroupName"),
            "vpc_id": get_str(sg, "VpcId"),
            "description": get_str(sg, "Description"),
            "ingress_rules": ingress_count,
            "egress_rules": egress_count,
        }));
    }

    // Network ACLs
    let nacls = aws_list(&["ec2", "describe-network-acls"], "NetworkAcls", region, profile)?;
    for nacl in &nacls {
        if b(nacl, "IsDefault") { continue; }
        let name = get_name(nacl);
        res.add("aws_network_acl", get_str(nacl, "NetworkAclId"), &name, serde_json::json!({
            "vpc_id": get_str(nacl, "VpcId"),
        }));
    }

    Ok(res)
}
