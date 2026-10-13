# MonMetreCarré

**Track real estate prices in France by commune**

MonMetreCarré is a web application that provides real-time real estate price data for French communes, including average square meter prices for apartments and houses, based on official government data.

## Features

- **Real estate price tracking**: View average prices per square meter for apartments and houses
- **Historical data**: Track price evolution from 2022 onwards
- **Interactive charts**: Visualize price trends over time
- **Commune-specific data**: Get detailed statistics for any French commune
- **Responsive design**: Works on desktop and mobile devices

## Architecture

The application uses a serverless architecture on AWS with the following components:

- **Frontend**: Rust-based AWS Lambda function serving HTML with embedded JavaScript
- **Data Pipeline**: AWS Glue jobs for ETL processing
- **Storage**: 
  - S3 for raw and processed data
  - DynamoDB for fast lookups
- **API**: AWS Lambda with CloudFront CDN
- **Infrastructure**: Terraform for infrastructure as code

### Data Flow

1. **Data Ingestion**: Glue job downloads official data from data.gouv.fr
2. **Data Transformation**: Glue job processes raw CSV files and calculates statistics
3. **Data Storage**: Processed data stored in DynamoDB with partition keys by commune code (code INSEE)
4. **API**: Lambda function queries DynamoDB and serves HTML pages with embedded data
5. **Frontend**: CloudFront distribution caches responses for fast delivery

## Project Structure

```
monmetrecarre/
├── etl/                    # ETL scripts (Python for Glue jobs)
│   ├── cities.py           # Loads French commune data
│   ├── ingest.py           # Downloads and processes raw data
│   └── transform.py        # Calculates statistics and loads to DynamoDB
├── lambda/
│   └── vf_front/           # Rust Lambda function for frontend
│       ├── src/           # Source code
│       │   ├── main.rs
│       │   └── http_handler.rs
│       ├── Cargo.toml     # Rust dependencies
│       └── templates/     # HTML templates
│           └── vf.html     # Main template with charts
├── terraform/             # Infrastructure as code
│   ├── cloudfront.tf      # CloudFront distribution
│   ├── dynamodb.tf        # DynamoDB tables
│   ├── glue.tf            # Glue jobs and crawlers
│   ├── iam.tf             # IAM roles and policies
│   ├── lambda.tf          # Lambda functions
│   ├── providers.tf       # AWS provider configuration
│   └── s3.tf              # S3 buckets
├── LICENSE                # MIT License
└── README.md              # This file
```

## Setup

### Prerequisites

- AWS account with appropriate permissions
- Terraform installed
- Rust toolchain (for Lambda function)
- AWS CLI configured with credentials

### Deployment

1. **Initialize Terraform**:
   ```bash
   cd terraform
   terraform init
   ```

2. **Review and apply infrastructure**:
   ```bash
   terraform plan
   terraform apply
   ```

3. **Build and deploy Lambda function**:
   ```bash
   cd lambda/vf_front
   cargo lambda build --release
   # Upload the binary to S3 or use cargo lambda deploy
   ```

4. **Run ETL jobs**:
   - Trigger the Glue jobs manually or set up a schedule
   - The jobs will download data, process it, and load it into DynamoDB

### Local Development

For local testing of the Rust Lambda function:

```bash
cd lambda/vf_front
cargo lambda watch
```

Then test with a sample payload:
```bash
cargo lambda invoke --data-file ./payload.json
```

## Data Sources

- **Real estate data**: [data.gouv.fr](https://www.data.gouv.fr/) - Official French government open data portal
- **Commune data**: [geo.api.gouv.fr](https://geo.api.gouv.fr/) - French geographic API

## API Endpoint

The application serves data via a single endpoint:
```
GET /{type}/{code_insee}
```

Where:
- `type`: Type of property (currently supports `codeInsee`)
- `code_insee`: The INSEE code of the French commune (5-digit code)

Example:
```
https://your-distribution.cloudfront.net/codeInsee/75056
```

This returns an HTML page with:
- Current average price per square meter for apartments and houses
- Historical price trends
- Interactive charts
- Statistics (average surface, number of transactions, etc.)

## Configuration

The application uses environment variables for the Lambda function:

- `VF_DYNAMODB_TABLE_NAME`: Name of the DynamoDB table for property data (default: `monmetrecarre`)
- `CITIES_DYNAMODB_TABLE_NAME`: Name of the DynamoDB table for commune data (default: `cities`)

## Technologies Used

- **Rust**: Primary language for Lambda function
- **Python**: ETL scripts for Glue jobs
- **Terraform**: Infrastructure as code
- **AWS Services**:
  - Lambda
  - API Gateway / CloudFront
  - DynamoDB
  - S3
  - Glue
  - IAM
- **Frontend**: HTML, CSS, JavaScript with Chart.js for visualization
- **Styling**: Tailwind CSS

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Acknowledgments

- French government for providing open data on real estate transactions
- AWS for their comprehensive serverless platform
