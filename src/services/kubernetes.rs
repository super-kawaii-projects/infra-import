use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("kubernetes");

    // EKS Clusters
    let clusters = aws_list(&["eks", "list-clusters"], "clusters", region, profile)?;
    for name_val in &clusters {
        if let Some(name) = name_val.as_str() {
            // Describe cluster for details
            let cluster_json = aws_cli(&["eks", "describe-cluster", "--name", name], region, profile)?;
            let c = &cluster_json["cluster"];
            res.add("aws_eks_cluster", name, name, serde_json::json!({
                "name": name,
                "version": get_str(c, "version"),
                "role_arn": get_str(c, "roleArn"),
            }));

            // Node Groups
            if let Ok(ngs) = aws_list(&["eks", "list-nodegroups", "--cluster-name", name], "nodegroups", region, profile) {
                for ng_val in &ngs {
                    if let Some(ng_name) = ng_val.as_str() {
                        let id = format!("{}:{}", name, ng_name);
                        res.add("aws_eks_node_group", &id, ng_name, serde_json::json!({
                            "cluster_name": name,
                            "node_group_name": ng_name,
                        }));
                    }
                }
            }

            // Addons
            if let Ok(addons) = aws_list(&["eks", "list-addons", "--cluster-name", name], "addons", region, profile) {
                for addon_val in &addons {
                    if let Some(addon) = addon_val.as_str() {
                        let id = format!("{}:{}", name, addon);
                        res.add("aws_eks_addon", &id, addon, serde_json::json!({
                            "cluster_name": name,
                            "addon_name": addon,
                        }));
                    }
                }
            }
        }
    }

    Ok(res)
}
