mod aws;
mod scanner;
mod generator;
mod resources;
mod services;

use clap::{Parser, ValueEnum};
use colored::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "infra-import")]
#[command(about = "Scan ALL existing AWS infrastructure and generate Terraform code")]
#[command(version, long_about = None)]
struct Cli {
    /// AWS region to scan
    #[arg(short, long, default_value = "us-east-1")]
    region: String,

    /// AWS profile to use
    #[arg(short, long)]
    profile: Option<String>,

    /// What to scan (default: everything)
    #[arg(short, long, value_enum, default_value = "all")]
    scope: Scope,

    /// Output directory
    #[arg(short, long, default_value = "./imported")]
    output: PathBuf,

    /// Filter by VPC ID
    #[arg(long)]
    vpc_id: Option<String>,

    /// Dry run — show what would be scanned
    #[arg(long)]
    dry_run: bool,

    /// Skip services that error (don't abort on permission denied)
    #[arg(long)]
    skip_errors: bool,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Scope {
    /// VPCs, Subnets, NATs, IGWs, TGWs, Route Tables, Endpoints
    Networking,
    /// EC2, ASGs, Launch Templates, Key Pairs
    Compute,
    /// ALBs, NLBs, Target Groups
    Loadbalancing,
    /// Security Groups, NACLs
    Security,
    /// IAM Users, Roles, Policies, Groups, Instance Profiles
    Iam,
    /// S3, EBS, EFS
    Storage,
    /// RDS, DynamoDB, ElastiCache
    Database,
    /// EKS clusters, Node Groups, Addons
    Kubernetes,
    /// Lambda functions
    Lambda,
    /// ECS clusters, Services
    Containers,
    /// SQS, SNS, Kinesis
    Messaging,
    /// CloudWatch Alarms, Log Groups
    Monitoring,
    /// Route53, ACM, CloudFront
    Dns,
    /// Secrets Manager, SSM Parameters, KMS Keys
    Secrets,
    /// ECR repositories
    Ecr,
    /// Everything — scan all services
    All,
}

impl Scope {
    pub fn includes(&self, target: &Scope) -> bool {
        matches!(self, Scope::All) || std::mem::discriminant(self) == std::mem::discriminant(target)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    print_banner();
    println!("Region:  {}", cli.region.yellow());
    if let Some(ref p) = cli.profile { println!("Profile: {}", p.yellow()); }
    println!("Scope:   {}", format!("{:?}", cli.scope).yellow());
    println!("Output:  {}", cli.output.display().to_string().yellow());
    println!();

    // Verify identity
    match aws::verify_identity(&cli.region, cli.profile.as_deref()) {
        Ok((account, arn)) => {
            println!("{}  Account:  {}", "✓".green(), account);
            println!("{}  Identity: {}", "✓".green(), arn);
            println!();
        }
        Err(e) => {
            eprintln!("{} Auth failed: {}", "✗".red(), e);
            std::process::exit(1);
        }
    }

    if cli.dry_run {
        println!("{}", "DRY RUN — services that would be scanned:".yellow().bold());
        print_scan_plan(&cli.scope);
        return Ok(());
    }

    // Scan everything
    println!("{}", "🔍 Scanning AWS resources...".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());

    let results = scanner::scan_all(&cli.region, cli.profile.as_deref(), &cli.scope, cli.vpc_id.as_deref(), cli.skip_errors);

    // Print summary
    println!();
    println!("{} Scan complete. {} total resources found.", "✓".green().bold(), results.total_count().to_string().bold());
    println!();
    results.print_full_summary();
    println!();

    // Generate terraform into organized directories
    println!("{}", "📝 Generating Terraform...".bold());
    generator::generate_all(&results, &cli.output, &cli.region)?;

    println!();
    println!("{} Output written to: {}", "✓".green().bold(), cli.output.display().to_string().bold());
    println!();
    print_output_tree(&cli.output);
    println!();
    println!("{}:", "Next steps".bold());
    println!("  cd {}", cli.output.display());
    println!("  terraform init");
    println!("  bash import.sh        {}", "← imports state for all resources".dimmed());
    println!("  terraform plan        {}", "← should show 0 changes if complete".dimmed());
    println!();

    Ok(())
}

fn print_banner() {
    println!("{}", r#"
  ___        __               _                            _
 |_ _|_ __  / _|_ __ __ _   (_)_ __ ___  _ __   ___  _ _| |_
  | || '_ \| |_| '__/ _` |  | | '_ ` _ \| '_ \ / _ \| '_|  _|
  | || | | |  _| | | (_| |  | | | | | | | |_) | (_) | |  | |_
 |___|_| |_|_| |_|  \__,_|  |_|_| |_| |_| .__/ \___/|_|   \__|
                                          |_|
"#.cyan());
}

fn print_scan_plan(scope: &Scope) {
    let services = get_services_for_scope(scope);
    for s in services {
        println!("  {} {}", "→".dimmed(), s);
    }
}

fn get_services_for_scope(scope: &Scope) -> Vec<&'static str> {
    match scope {
        Scope::All => vec![
            "EC2 (instances, key pairs)",
            "VPC (VPCs, subnets, NATs, IGWs, TGWs, route tables, endpoints)",
            "Security Groups & NACLs",
            "ELB (ALBs, NLBs, target groups)",
            "Auto Scaling Groups & Launch Templates",
            "EKS (clusters, node groups, addons)",
            "ECS (clusters, services)",
            "ECR (repositories)",
            "Lambda (functions)",
            "RDS (instances)",
            "DynamoDB (tables)",
            "ElastiCache (replication groups)",
            "S3 (buckets)",
            "EBS (volumes)",
            "EFS (filesystems)",
            "IAM (roles, policies, users, groups)",
            "KMS (keys)",
            "Secrets Manager (secrets)",
            "SSM (parameters)",
            "Route53 (hosted zones)",
            "ACM (certificates)",
            "CloudFront (distributions)",
            "SQS (queues)",
            "SNS (topics)",
            "Kinesis (streams)",
            "CloudWatch (alarms, log groups)",
        ],
        Scope::Networking => vec!["VPCs", "Subnets", "NATs", "IGWs", "TGWs", "Route Tables", "Endpoints", "EIPs"],
        Scope::Compute => vec!["EC2 Instances", "Key Pairs", "ASGs", "Launch Templates"],
        Scope::Iam => vec!["Roles", "Policies", "Users", "Groups", "Instance Profiles"],
        Scope::Storage => vec!["S3 Buckets", "EBS Volumes", "EFS Filesystems"],
        Scope::Database => vec!["RDS", "DynamoDB", "ElastiCache"],
        Scope::Kubernetes => vec!["EKS Clusters", "Node Groups", "Addons"],
        Scope::Lambda => vec!["Functions"],
        Scope::Containers => vec!["ECS Clusters", "Services"],
        Scope::Messaging => vec!["SQS Queues", "SNS Topics", "Kinesis Streams"],
        Scope::Monitoring => vec!["CloudWatch Alarms", "Log Groups"],
        Scope::Dns => vec!["Route53 Zones", "ACM Certificates", "CloudFront Distributions"],
        Scope::Secrets => vec!["Secrets Manager", "SSM Parameters", "KMS Keys"],
        Scope::Security => vec!["Security Groups", "NACLs"],
        Scope::Loadbalancing => vec!["ALBs", "NLBs", "Target Groups"],
        Scope::Ecr => vec!["Repositories"],
    }
}

fn print_output_tree(output: &Path) {
    println!("  {}/", output.display().to_string().bold());
    let dirs = ["networking", "compute", "loadbalancing", "security",
        "iam", "storage", "database", "kubernetes", "lambda",
        "containers", "messaging", "monitoring", "dns", "secrets", "ecr"];
    for d in dirs {
        let path = output.join(d);
        if path.exists() {
            let count = std::fs::read_dir(&path).map(|r| r.count()).unwrap_or(0);
            if count > 0 {
                println!("  ├── {}/ ({} files)", d, count);
            }
        }
    }
    println!("  ├── provider.tf");
    println!("  └── import.sh");
}
