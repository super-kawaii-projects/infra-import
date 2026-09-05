use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("secrets");

    // Secrets Manager
    let secrets = aws_list(&["secretsmanager", "list-secrets"], "SecretList", region, profile)?;
    for s_val in &secrets {
        let name = get_str(s_val, "Name");
        let arn = get_str(s_val, "ARN");
        res.add("aws_secretsmanager_secret", arn, name, serde_json::json!({
            "name": name,
        }));
    }

    // SSM Parameters
    let params = aws_list(&["ssm", "describe-parameters"], "Parameters", region, profile)?;
    for p in &params {
        let name = get_str(p, "Name");
        let ptype = get_str(p, "Type");
        res.add("aws_ssm_parameter", name, name, serde_json::json!({
            "name": name,
            "type": ptype,
        }));
    }

    // KMS Keys (customer-managed)
    let keys = aws_list(&["kms", "list-keys"], "Keys", region, profile)?;
    for k in &keys {
        let key_id = get_str(k, "KeyId");
        // Describe to filter out AWS-managed keys
        if let Ok(desc) = aws_cli(&["kms", "describe-key", "--key-id", key_id], region, profile) {
            let meta = &desc["KeyMetadata"];
            let mgr = get_str(meta, "KeyManager");
            if mgr == "CUSTOMER" {
                let description = get_str(meta, "Description");
                res.add("aws_kms_key", key_id, key_id, serde_json::json!({
                    "key_id": key_id,
                    "description": description,
                }));
            }
        }
    }

    Ok(res)
}
