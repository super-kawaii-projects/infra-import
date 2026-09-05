use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("compute");

    // EC2 Instances
    let reservations = aws_list(&["ec2", "describe-instances"], "Reservations", region, profile)?;
    for r in &reservations {
        let instances = r.get("Instances").and_then(|v| v.as_array());
        if let Some(instances) = instances {
            for i in instances {
                let name = get_name(i);
                let sg_ids: Vec<String> = i.get("SecurityGroups")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(|g| s(g, "GroupId")).collect())
                    .unwrap_or_default();
                res.add("aws_instance", get_str(i, "InstanceId"), &name, serde_json::json!({
                    "instance_type": get_str(i, "InstanceType"),
                    "ami": get_str(i, "ImageId"),
                    "subnet_id": get_str(i, "SubnetId"),
                    "key_name": get_str(i, "KeyName"),
                    "vpc_security_group_ids": sg_ids,
                }));
            }
        }
    }

    // Key Pairs
    let keys = aws_list(&["ec2", "describe-key-pairs"], "KeyPairs", region, profile)?;
    for k in &keys {
        res.add("aws_key_pair", get_str(k, "KeyPairId"), get_str(k, "KeyName"), serde_json::json!({
            "key_name": get_str(k, "KeyName"),
        }));
    }

    // Auto Scaling Groups
    let asgs = aws_list(&["autoscaling", "describe-auto-scaling-groups"], "AutoScalingGroups", region, profile)?;
    for a in &asgs {
        let name = get_str(a, "AutoScalingGroupName");
        let lt_id = a.get("LaunchTemplate")
            .and_then(|lt| lt.get("LaunchTemplateId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        res.add("aws_autoscaling_group", name, name, serde_json::json!({
            "min_size": i(a, "MinSize"),
            "max_size": i(a, "MaxSize"),
            "desired_capacity": i(a, "DesiredCapacity"),
            "launch_template": lt_id,
        }));
    }

    // Launch Templates
    let lts = aws_list(&["ec2", "describe-launch-templates"], "LaunchTemplates", region, profile)?;
    for lt in &lts {
        res.add("aws_launch_template", get_str(lt, "LaunchTemplateId"), get_str(lt, "LaunchTemplateName"), serde_json::json!({
            "name": get_str(lt, "LaunchTemplateName"),
        }));
    }

    Ok(res)
}
