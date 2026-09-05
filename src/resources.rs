use colored::*;
use serde_json::Value;

/// A single discovered AWS resource
#[derive(Debug, Clone)]
pub struct Resource {
    pub tf_type: String,
    pub id: String,
    pub name: String,
    pub attributes: Value,
}

/// Resources discovered from a single service scan
#[derive(Debug)]
pub struct ServiceResources {
    pub service_name: String,
    pub resources: Vec<Resource>,
}

impl ServiceResources {
    pub fn new(name: &str) -> Self {
        Self { service_name: name.to_string(), resources: Vec::new() }
    }

    pub fn add(&mut self, tf_type: &str, id: &str, name: &str, attrs: Value) {
        self.resources.push(Resource {
            tf_type: tf_type.to_string(),
            id: id.to_string(),
            name: name.to_string(),
            attributes: attrs,
        });
    }

    pub fn count(&self) -> usize { self.resources.len() }
}

/// All resources from all services
#[derive(Debug, Default)]
pub struct AllResources {
    pub services: Vec<ServiceResources>,
}

impl AllResources {
    pub fn total_count(&self) -> usize {
        self.services.iter().map(|s| s.count()).sum()
    }

    pub fn add_service(&mut self, svc: ServiceResources) {
        if svc.count() > 0 {
            self.services.push(svc);
        }
    }

    pub fn print_full_summary(&self) {
        for svc in &self.services {
            if svc.count() > 0 {
                println!("  {} {}: {} resources",
                    "│".dimmed(),
                    svc.service_name.bold(),
                    svc.count().to_string().green()
                );
                // Group by tf_type
                let mut type_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                for r in &svc.resources {
                    *type_counts.entry(&r.tf_type).or_insert(0) += 1;
                }
                for (tf_type, count) in &type_counts {
                    println!("  {}   {:>3} {}", "│".dimmed(), count, tf_type.dimmed());
                }
            }
        }
    }
}
