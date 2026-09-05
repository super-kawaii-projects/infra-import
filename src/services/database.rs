use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("database");

    // RDS Instances
    let dbs = aws_list(&["rds", "describe-db-instances"], "DBInstances", region, profile)?;
    for db in &dbs {
        let id = get_str(db, "DBInstanceIdentifier");
        res.add("aws_db_instance", id, id, serde_json::json!({
            "engine": get_str(db, "Engine"),
            "engine_version": get_str(db, "EngineVersion"),
            "instance_class": get_str(db, "DBInstanceClass"),
            "allocated_storage": i(db, "AllocatedStorage"),
            "multi_az": b(db, "MultiAZ"),
            "storage_encrypted": b(db, "StorageEncrypted"),
        }));
    }

    // DynamoDB Tables
    let tables = aws_list(&["dynamodb", "list-tables"], "TableNames", region, profile)?;
    for name_val in &tables {
        if let Some(name) = name_val.as_str() {
            res.add("aws_dynamodb_table", name, name, serde_json::json!({ "name": name }));
        }
    }

    // ElastiCache Replication Groups
    let caches = aws_list(&["elasticache", "describe-replication-groups"], "ReplicationGroups", region, profile)?;
    for rg in &caches {
        let id = get_str(rg, "ReplicationGroupId");
        res.add("aws_elasticache_replication_group", id, id, serde_json::json!({
            "description": get_str(rg, "Description"),
        }));
    }

    Ok(res)
}
