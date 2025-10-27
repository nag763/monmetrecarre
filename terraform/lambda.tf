resource "aws_lambda_function" "vf_stats_lambda" {
  function_name    = "monmetrecarre-vf-stats"
  handler          = "index.handler" # Assuming main.py and handler function
  runtime          = "nodejs22.x"
  role             = aws_iam_role.lambda_vf_stats_role.arn
  filename         = "../lambda/vf_stats/function.zip"
  source_code_hash = filebase64sha256("../lambda/vf_stats/function.zip")
  timeout          = 30 # seconds

  environment {
    variables = {
      DYNAMODB_TABLE_NAME = aws_dynamodb_table.monmetrecarre_dynamodb_table.name
    }
  }

  tags = {
    Name = "monmetrecarre"
  }
}