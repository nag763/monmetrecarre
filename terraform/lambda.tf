resource "random_string" "random" {
  length           = 128
  special          = true
}

resource "aws_lambda_function" "vf_stats_lambda" {
  function_name    = "monmetrecarre-vf-stats"
  handler          = "index.handler" # Assuming main.py and handler function
  runtime          = "nodejs22.x"
  role             = aws_iam_role.lambda_vf_stats_role.arn
  filename         = "../lambda/vf_stats/function.zip"
  source_code_hash = filebase64sha256("../lambda/vf_stats/function.zip")
  timeout          = 5

  environment {
    variables = {
      DYNAMODB_TABLE_NAME = aws_dynamodb_table.monmetrecarre_dynamodb_table.name
    }
  }

  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_lambda_permission" "allow_api_gateway_to_invoke_lambda" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.vf_stats_lambda.function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_apigatewayv2_api.monmetrecarre_api.execution_arn}/*/*"
}

resource "aws_lambda_function" "lambda_authorizer" {
  function_name    = "monmetrecarre-authorizer"
  handler          = "index.handler"
  runtime          = "nodejs22.x"
  role             = aws_iam_role.lambda_authorizer_role.arn
  filename         = "../lambda/auth/function.zip"
  source_code_hash = filebase64sha256("../lambda/auth/function.zip")
  timeout          = 5
  
  environment {
    variables = {
      SECRET_HEADER_VALUE = random_string.random.result
    }
  }

  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_lambda_permission" "allow_api_gateway_to_invoke_authorizer" {
  statement_id  = "AllowAPIGatewayInvokeAuthorizer"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.lambda_authorizer.function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_apigatewayv2_api.monmetrecarre_api.execution_arn}/*/*"
}