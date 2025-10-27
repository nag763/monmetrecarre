# Glue Database
resource "aws_glue_catalog_database" "monmetrecarre_glue_database" {
  name = "monmetrecarre_database"

  tags = {
    Name = "monmetrecarre"
  }
}

# Glue Job 1: Ingest data from API to S3 raw
resource "aws_glue_job" "glue_job_api_to_raw" {
  name     = "monmetrecarre-api-to-raw-job"
  role_arn = aws_iam_role.glue_job_role.arn

  command {
    script_location = "s3://${aws_s3_bucket.monmetrecarre_bucket.id}/scripts/ingest.py"
    python_version  = "3"
  }

  default_arguments = {
    "--TempDir"             = "s3://${aws_s3_object.temporary.id}",
    "--temp_dir_path"       = "temporary",
    "--bucket_name"         = aws_s3_bucket.monmetrecarre_bucket.id
    "--compressed_prefix"   = "raw"
    "--uncompressed_prefix" = "transformed"
  }

  worker_type       = "G.1X"
  number_of_workers = 4
  glue_version      = "3.0"

  tags = {
    Name = "monmetrecarre"
  }
}

# Glue Crawler: Crawl S3 raw object and publish to Glue table
resource "aws_glue_crawler" "monmetrecarre_raw_crawler" {
  database_name = aws_glue_catalog_database.monmetrecarre_glue_database.name
  name          = "monmetrecarre-raw-crawler"
  role          = aws_iam_role.glue_crawler_role.arn
  s3_target {
    path = "s3://${aws_s3_bucket.monmetrecarre_bucket.id}/transformed/vf/"
  }

  tags = {
    Name = "monmetrecarre"
  }
}

# Glue Job 2: Read from Glue table and publish to DynamoDB
resource "aws_glue_job" "glue_job_raw_to_dynamodb" {
  name     = "monmetrecarre-raw-to-dynamodb-job"
  role_arn = aws_iam_role.glue_job_role.arn
  command {
    script_location = "s3://${aws_s3_bucket.monmetrecarre_bucket.id}/scripts/transform.py"
    python_version  = "3"
  }
  default_arguments = {
    "--TempDir"             = "s3://${aws_s3_bucket.monmetrecarre_bucket.id}/temporary/",
    "--dynamodb_table_name" = aws_dynamodb_table.monmetrecarre_dynamodb_table.name,
    "--glue_database"       = aws_glue_catalog_database.monmetrecarre_glue_database.name,
    "--glue_table"          = "vf",
  }
  worker_type       = "G.1X"
  number_of_workers = 4
  glue_version      = "5.0"

  tags = {
    Name = "monmetrecarre"
  }
}
