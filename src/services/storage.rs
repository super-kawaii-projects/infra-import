use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("storage");

    // S3 Buckets
    let buckets = aws_list(&["s3api", "list-buckets"], "Buckets", region, profile)?;
    for b in &buckets {
        let name = get_str(b, "Name");
        res.add("aws_s3_bucket", name, name, serde_json::json!({
            "bucket": name,
        }));
    }

    // EBS Volumes
    let volumes = aws_list(&["ec2", "describe-volumes"], "Volumes", region, profile)?;
    for v in &volumes {
        let name = get_name(v);
        res.add("aws_ebs_volume", get_str(v, "VolumeId"), &name, serde_json::json!({
            "size": i(v, "Size"),
            "type": get_str(v, "VolumeType"),
            "encrypted": self::b(v, "Encrypted"),
            "availability_zone": get_str(v, "AvailabilityZone"),
        }));
    }

    // EFS File Systems
    let filesystems = aws_list(&["efs", "describe-file-systems"], "FileSystems", region, profile)?;
    for f in &filesystems {
        let name_val = f.get("Name").and_then(|v| v.as_str()).unwrap_or_default();
        res.add("aws_efs_file_system", get_str(f, "FileSystemId"), name_val, serde_json::json!({
            "encrypted": self::b(f, "Encrypted"),
            "performance_mode": get_str(f, "PerformanceMode"),
            "throughput_mode": get_str(f, "ThroughputMode"),
        }));
    }

    Ok(res)
}

fn b(val: &serde_json::Value, key: &str) -> bool {
    val.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}
