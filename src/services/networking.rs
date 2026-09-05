use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>, vpc_filter: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("networking");

    // VPCs
    let vpcs = aws_list(&["ec2", "describe-vpcs"], "Vpcs", region, profile)?;
    for v in &vpcs {
        let id = get_str(v, "VpcId");
        if let Some(filter) = vpc_filter { if id != filter { continue; } }
        res.add("aws_vpc", id, &get_name(v), serde_json::json!({
            "cidr_block": get_str(v, "CidrBlock"),
            "enable_dns_hostnames": true,
            "enable_dns_support": true,
            "tags": extract_tags(v),
        }));
    }

    // Subnets
    let subnets = aws_list(&["ec2", "describe-subnets"], "Subnets", region, profile)?;
    for s in &subnets {
        let vpc = get_str(s, "VpcId");
        if let Some(filter) = vpc_filter { if vpc != filter { continue; } }
        res.add("aws_subnet", get_str(s, "SubnetId"), &get_name(s), serde_json::json!({
            "vpc_id": vpc,
            "cidr_block": get_str(s, "CidrBlock"),
            "availability_zone": get_str(s, "AvailabilityZone"),
            "map_public_ip_on_launch": b(s, "MapPublicIpOnLaunch"),
            "tags": extract_tags(s),
        }));
    }

    // NAT Gateways
    let nats = aws_list(&["ec2", "describe-nat-gateways"], "NatGateways", region, profile)?;
    for n in &nats {
        let alloc = n.get("NatGatewayAddresses")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.get("AllocationId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        res.add("aws_nat_gateway", get_str(n, "NatGatewayId"), &get_name(n), serde_json::json!({
            "subnet_id": get_str(n, "SubnetId"),
            "allocation_id": alloc,
            "tags": extract_tags(n),
        }));
    }

    // Internet Gateways
    let igws = aws_list(&["ec2", "describe-internet-gateways"], "InternetGateways", region, profile)?;
    for igw in &igws {
        let vpc = igw.get("Attachments")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|a| a.get("VpcId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        res.add("aws_internet_gateway", get_str(igw, "InternetGatewayId"), &get_name(igw), serde_json::json!({
            "vpc_id": vpc,
            "tags": extract_tags(igw),
        }));
    }

    // Transit Gateways
    let tgws = aws_list(&["ec2", "describe-transit-gateways"], "TransitGateways", region, profile)?;
    for t in &tgws {
        res.add("aws_ec2_transit_gateway", get_str(t, "TransitGatewayId"), &get_name(t), serde_json::json!({
            "tags": extract_tags(t),
        }));
    }

    // Route Tables
    let rts = aws_list(&["ec2", "describe-route-tables"], "RouteTables", region, profile)?;
    for rt in &rts {
        let vpc = get_str(rt, "VpcId");
        if let Some(filter) = vpc_filter { if vpc != filter { continue; } }
        res.add("aws_route_table", get_str(rt, "RouteTableId"), &get_name(rt), serde_json::json!({
            "vpc_id": vpc,
            "tags": extract_tags(rt),
        }));
    }

    // VPC Endpoints
    let eps = aws_list(&["ec2", "describe-vpc-endpoints"], "VpcEndpoints", region, profile)?;
    for ep in &eps {
        res.add("aws_vpc_endpoint", get_str(ep, "VpcEndpointId"), get_str(ep, "ServiceName"), serde_json::json!({
            "vpc_id": get_str(ep, "VpcId"),
            "service_name": get_str(ep, "ServiceName"),
        }));
    }

    // Elastic IPs
    let eips = aws_list(&["ec2", "describe-addresses"], "Addresses", region, profile)?;
    for eip in &eips {
        res.add("aws_eip", get_str(eip, "AllocationId"), &get_name(eip), serde_json::json!({
            "public_ip": get_str(eip, "PublicIp"),
            "domain": get_str(eip, "Domain"),
        }));
    }

    Ok(res)
}
