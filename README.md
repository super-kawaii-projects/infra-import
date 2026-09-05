# infra-import

Scan your existing AWS infrastructure and generate Terraform code + import scripts. One command, all your resources, organized into clean directories.

## What it does

```
infra-import --region us-east-1
```

Produces:

```
imported/
├── networking/networking.tf    # VPCs, subnets, NATs, IGWs, route tables
├── compute/compute.tf          # EC2 instances, ASGs, launch templates
├── loadbalancing/loadbalancing.tf
├── security/security.tf        # Security groups, NACLs
├── iam/iam.tf                  # Roles, policies, users, groups
├── storage/storage.tf          # S3 buckets, EBS volumes, EFS
├── database/database.tf        # RDS, DynamoDB, ElastiCache
├── kubernetes/kubernetes.tf    # EKS clusters, node groups, addons
├── lambda/lambda.tf
├── containers/containers.tf    # ECS clusters, services
├── messaging/messaging.tf      # SQS, SNS, Kinesis
├── monitoring/monitoring.tf    # CloudWatch alarms, log groups
├── dns/dns.tf                  # Route53, ACM, CloudFront
├── secrets/secrets.tf          # Secrets Manager, SSM, KMS
├── ecr/ecr.tf
├── provider.tf
└── import.sh                   # terraform import commands for every resource
```

Then:

```bash
cd imported
terraform init
bash import.sh          # imports all resources into state
terraform plan          # should show 0 changes
```

## Install

### Quick install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/infra-import/main/install.sh | bash
```

### GitHub Releases

Download the binary for your platform from [Releases](https://github.com/OWNER/infra-import/releases/latest):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `infra-import-linux-amd64` |
| Linux ARM64 | `infra-import-linux-arm64` |
| macOS Intel | `infra-import-darwin-amd64` |
| macOS Apple Silicon | `infra-import-darwin-arm64` |
| Windows | `infra-import-windows-amd64.exe` |

### Docker

```bash
docker run --rm -v ~/.aws:/root/.aws -v ./imported:/output \
  ghcr.io/OWNER/infra-import --region us-east-1 --output /output
```

### From source

```bash
git clone https://github.com/OWNER/infra-import.git
cd infra-import
cargo install --path .
```

## Prerequisites

- **AWS CLI** installed and configured (`aws configure`)
- Valid AWS credentials with read access to the services you want to scan

## Usage

```bash
# Scan everything in us-east-1
infra-import --region us-east-1

# Use a specific AWS profile
infra-import --region us-east-1 --profile production

# Scan only networking resources
infra-import --scope networking --region us-east-1

# Scan only resources in a specific VPC
infra-import --region us-east-1 --vpc-id vpc-abc123

# Dry run — see what would be scanned
infra-import --dry-run

# Skip services you don't have permissions for
infra-import --region us-east-1 --skip-errors

# Custom output directory
infra-import --region us-east-1 --output ./terraform/imported
```

## Scopes

| Scope | Resources |
|-------|-----------|
| `networking` | VPCs, Subnets, NATs, IGWs, TGWs, Route Tables, VPC Endpoints, EIPs |
| `compute` | EC2 Instances, Key Pairs, ASGs, Launch Templates |
| `loadbalancing` | ALBs, NLBs, Target Groups |
| `security` | Security Groups, NACLs |
| `iam` | Roles, Policies, Users, Groups, Instance Profiles |
| `storage` | S3 Buckets, EBS Volumes, EFS |
| `database` | RDS, DynamoDB, ElastiCache |
| `kubernetes` | EKS Clusters, Node Groups, Addons |
| `lambda` | Lambda Functions |
| `containers` | ECS Clusters, Services |
| `messaging` | SQS Queues, SNS Topics, Kinesis Streams |
| `monitoring` | CloudWatch Alarms, Log Groups |
| `dns` | Route53 Zones, ACM Certificates, CloudFront |
| `secrets` | Secrets Manager, SSM Parameters, KMS Keys |
| `ecr` | ECR Repositories |
| `all` | Everything (default) |

## How it works

1. Calls AWS CLI commands to list resources in your account
2. Parses the JSON responses
3. Generates Terraform resource blocks with attributes
4. Writes organized `.tf` files by service category
5. Generates an `import.sh` script with `terraform import` commands for every discovered resource

## IAM Permissions

The tool needs read-only access. A policy like `ReadOnlyAccess` works, or you can scope it to specific services. At minimum:

```json
{
  "Effect": "Allow",
  "Action": [
    "ec2:Describe*",
    "eks:List*", "eks:Describe*",
    "rds:Describe*",
    "s3:ListAllMyBuckets",
    "iam:List*",
    "lambda:ListFunctions",
    "ecs:List*",
    "elasticloadbalancing:Describe*",
    "autoscaling:Describe*",
    "dynamodb:ListTables",
    "elasticache:Describe*",
    "ecr:DescribeRepositories",
    "sqs:ListQueues",
    "sns:ListTopics",
    "kinesis:ListStreams",
    "cloudwatch:DescribeAlarms",
    "logs:DescribeLogGroups",
    "route53:ListHostedZones",
    "acm:ListCertificates",
    "cloudfront:ListDistributions",
    "secretsmanager:ListSecrets",
    "ssm:DescribeParameters",
    "kms:ListKeys", "kms:DescribeKey",
    "sts:GetCallerIdentity"
  ],
  "Resource": "*"
}
```

## License

Copyright (c) 2026 Stillwater Strategic Solutions LLC. All rights reserved.

This is **source-available** software, not open source. It is free for
personal, non-commercial evaluation and testing. **Commercial and production
use requires a paid license.** See [LICENSE](LICENSE) for full terms, or
contact Stillwater Strategic Solutions LLC at michaelisaacs121092@gmail.com
for commercial licensing.
