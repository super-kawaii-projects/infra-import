use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("loadbalancing");

    // Load Balancers (ALB + NLB)
    let lbs = aws_list(&["elbv2", "describe-load-balancers"], "LoadBalancers", region, profile)?;
    for lb in &lbs {
        let lb_type = get_str(lb, "Type");
        res.add("aws_lb", get_str(lb, "LoadBalancerArn"), get_str(lb, "LoadBalancerName"), serde_json::json!({
            "name": get_str(lb, "LoadBalancerName"),
            "type": lb_type,
            "scheme": get_str(lb, "Scheme"),
            "dns_name": get_str(lb, "DNSName"),
            "vpc_id": get_str(lb, "VpcId"),
        }));
    }

    // Target Groups
    let tgs = aws_list(&["elbv2", "describe-target-groups"], "TargetGroups", region, profile)?;
    for tg in &tgs {
        res.add("aws_lb_target_group", get_str(tg, "TargetGroupArn"), get_str(tg, "TargetGroupName"), serde_json::json!({
            "name": get_str(tg, "TargetGroupName"),
            "port": i(tg, "Port"),
            "protocol": get_str(tg, "Protocol"),
            "vpc_id": get_str(tg, "VpcId"),
        }));
    }

    Ok(res)
}
