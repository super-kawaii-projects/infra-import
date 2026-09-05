use crate::aws::*;
use crate::resources::*;
use anyhow::Result;

pub fn scan(region: &str, profile: Option<&str>) -> Result<ServiceResources> {
    let mut res = ServiceResources::new("ecr");

    let repos = aws_list(&["ecr", "describe-repositories"], "repositories", region, profile)?;
    for r in &repos {
        let name = get_str(r, "repositoryName");
        res.add("aws_ecr_repository", name, name, serde_json::json!({
            "name": name,
            "uri": get_str(r, "repositoryUri"),
            "image_tag_mutability": get_str(r, "imageTagMutability"),
        }));
    }

    Ok(res)
}
