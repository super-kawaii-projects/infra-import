use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("dns");

    // Route53 Hosted Zones
    let zones = aws_list(&["route53", "list-hosted-zones"], "HostedZones", region, profile)?;
    for z in &zones {
        let raw_id = get_str(z, "Id");
        let id = raw_id.trim_start_matches("/hostedzone/");
        let name = get_str(z, "Name");
        let private = z.get("Config")
            .and_then(|c| c.get("PrivateZone"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        res.add("aws_route53_zone", id, name, serde_json::json!({
            "name": name,
            "private_zone": private,
        }));
    }

    // ACM Certificates
    let certs = aws_list(&["acm", "list-certificates"], "CertificateSummaryList", region, profile)?;
    for c in &certs {
        res.add("aws_acm_certificate", get_str(c, "CertificateArn"), get_str(c, "DomainName"), serde_json::json!({
            "domain_name": get_str(c, "DomainName"),
            "status": get_str(c, "Status"),
        }));
    }

    // CloudFront Distributions
    let dist_json = aws_cli(&["cloudfront", "list-distributions"], region, profile)?;
    let items = dist_json.get("DistributionList")
        .and_then(|dl| dl.get("Items"))
        .and_then(|v| v.as_array());
    if let Some(dists) = items {
        for d in dists {
            let comment = get_str(d, "Comment");
            res.add("aws_cloudfront_distribution", get_str(d, "Id"), comment, serde_json::json!({
                "domain_name": get_str(d, "DomainName"),
                "enabled": b(d, "Enabled"),
            }));
        }
    }

    Ok(res)
}
