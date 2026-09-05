use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("containers");

    // ECS Clusters
    let clusters = aws_list(&["ecs", "list-clusters"], "clusterArns", region, profile)?;
    for arn_val in &clusters {
        if let Some(arn) = arn_val.as_str() {
            let name = arn.split('/').last().unwrap_or(arn);
            res.add("aws_ecs_cluster", arn, name, serde_json::json!({ "name": name }));

            // ECS Services per cluster
            if let Ok(svcs) = aws_list(&["ecs", "list-services", "--cluster", arn], "serviceArns", region, profile) {
                for svc_val in &svcs {
                    if let Some(svc_arn) = svc_val.as_str() {
                        let svc_name = svc_arn.split('/').last().unwrap_or(svc_arn);
                        res.add("aws_ecs_service", svc_arn, svc_name, serde_json::json!({
                            "cluster": arn,
                            "name": svc_name,
                        }));
                    }
                }
            }
        }
    }

    Ok(res)
}
