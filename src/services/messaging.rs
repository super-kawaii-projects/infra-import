use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("messaging");

    // SQS Queues
    let queues = aws_list(&["sqs", "list-queues"], "QueueUrls", region, profile)?;
    for url_val in &queues {
        if let Some(url) = url_val.as_str() {
            let name = url.split('/').last().unwrap_or(url);
            res.add("aws_sqs_queue", url, name, serde_json::json!({ "name": name, "url": url }));
        }
    }

    // SNS Topics
    let topics = aws_list(&["sns", "list-topics"], "Topics", region, profile)?;
    for t in &topics {
        let arn = get_str(t, "TopicArn");
        let name = arn.split(':').last().unwrap_or(arn);
        res.add("aws_sns_topic", arn, name, serde_json::json!({ "name": name }));
    }

    // Kinesis Streams
    let streams = aws_list(&["kinesis", "list-streams"], "StreamNames", region, profile)?;
    for name_val in &streams {
        if let Some(name) = name_val.as_str() {
            res.add("aws_kinesis_stream", name, name, serde_json::json!({ "name": name }));
        }
    }

    Ok(res)
}
