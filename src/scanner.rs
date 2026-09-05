use colored::*;
use crate::resources::*;
use crate::services;
use crate::Scope;

pub fn scan_all(
    region: &str,
    profile: Option<&str>,
    scope: &Scope,
    vpc_filter: Option<&str>,
    skip_errors: bool,
) -> AllResources {
    let mut all = AllResources::default();

    if scope.includes(&Scope::Networking) {
        run_scan(&mut all, "Networking", services::networking::scan(region, profile, vpc_filter), skip_errors);
    }
    if scope.includes(&Scope::Compute) {
        run_scan(&mut all, "Compute", services::compute::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Loadbalancing) {
        run_scan(&mut all, "Load Balancing", services::loadbalancing::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Security) {
        run_scan(&mut all, "Security", services::security::scan(region, profile, vpc_filter), skip_errors);
    }
    if scope.includes(&Scope::Iam) {
        run_scan(&mut all, "IAM", services::iam::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Storage) {
        run_scan(&mut all, "Storage", services::storage::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Database) {
        run_scan(&mut all, "Database", services::database::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Kubernetes) {
        run_scan(&mut all, "Kubernetes", services::kubernetes::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Lambda) {
        run_scan(&mut all, "Lambda", services::lambda::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Containers) {
        run_scan(&mut all, "Containers", services::containers::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Messaging) {
        run_scan(&mut all, "Messaging", services::messaging::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Monitoring) {
        run_scan(&mut all, "Monitoring", services::monitoring::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Dns) {
        run_scan(&mut all, "DNS", services::dns::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Secrets) {
        run_scan(&mut all, "Secrets", services::secrets::scan(region, profile), skip_errors);
    }
    if scope.includes(&Scope::Ecr) {
        run_scan(&mut all, "ECR", services::ecr::scan(region, profile), skip_errors);
    }

    all
}

fn run_scan(all: &mut AllResources, label: &str, result: anyhow::Result<ServiceResources>, skip_errors: bool) {
    match result {
        Ok(svc) => {
            let count = svc.count();
            if count > 0 {
                println!("  {} {} — {} resources", "✓".green(), label, count);
            } else {
                println!("  {} {} — 0 resources", "·".dimmed(), label.dimmed());
            }
            all.add_service(svc);
        }
        Err(e) => {
            if skip_errors {
                println!("  {} {} — skipped: {}", "⚠".yellow(), label, format!("{}", e).dimmed());
            } else {
                println!("  {} {} — ERROR: {}", "✗".red(), label, e);
            }
        }
    }
}
