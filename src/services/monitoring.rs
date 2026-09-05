use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("monitoring");

    // CloudWatch Alarms
    let alarms = aws_list(&["cloudwatch", "describe-alarms"], "MetricAlarms", region, profile)?;
    for a in &alarms {
        let name = get_str(a, "AlarmName");
        res.add("aws_cloudwatch_metric_alarm", name, name, serde_json::json!({
            "alarm_name": name,
            "metric_name": get_str(a, "MetricName"),
            "namespace": get_str(a, "Namespace"),
        }));
    }

    // Log Groups
    let lgs = aws_list(&["logs", "describe-log-groups"], "logGroups", region, profile)?;
    for lg in &lgs {
        let name = get_str(lg, "logGroupName");
        res.add("aws_cloudwatch_log_group", name, name, serde_json::json!({
            "name": name,
            "retention_in_days": i(lg, "retentionInDays"),
        }));
    }

    Ok(res)
}
