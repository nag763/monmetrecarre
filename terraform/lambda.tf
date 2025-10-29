data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file = "../lambda/vf_front/target/x86_64-unknown-linux-musl/release/bootstrap"
  output_path = "/tmp/vf_stats_lambda.zip"
}

resource "aws_lambda_function" "vf_stats_lambda" {
  function_name    = "monmetrecarre-vf-stats"
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  role             = aws_iam_role.lambda_vf_stats_role.arn
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
  memory_size      = 128
  timeout          = 3

  environment {
    variables = {
      DYNAMODB_TABLE_NAME = aws_dynamodb_table.monmetrecarre_dynamodb_table.name
    }
  }

  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_lambda_function_url" "vf_stats_lambda_url" {
  function_name      = aws_lambda_function.vf_stats_lambda.function_name
  authorization_type = "NONE"
}