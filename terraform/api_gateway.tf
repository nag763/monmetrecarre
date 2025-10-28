resource "aws_apigatewayv2_api" "monmetrecarre_api" {
  name          = "monmetrecarre-http-api"
  protocol_type = "HTTP"
  description   = "HTTP API for monmetrecarre Lambda functions"
}

resource "aws_apigatewayv2_authorizer" "monmetrecarre_authorizer" {
  api_id = aws_apigatewayv2_api.monmetrecarre_api.id
  name   = "monmetrecarre-header-authorizer"
  authorizer_type = "REQUEST"  # HTTP APIs only support REQUEST type

  authorizer_uri = aws_lambda_function.lambda_authorizer.invoke_arn
  identity_sources = ["$request.header.x-origin-verify"]

  authorizer_payload_format_version = "2.0"  # Needed for { isAuthorized } format
  enable_simple_responses           = true   # This lets you use your new handler format
}

resource "aws_apigatewayv2_integration" "vf_stats_integration" {
  api_id = aws_apigatewayv2_api.monmetrecarre_api.id

  integration_type        = "AWS_PROXY"
  integration_uri         = aws_lambda_function.vf_stats_lambda.invoke_arn
  integration_method      = "POST"
  payload_format_version  = "2.0"
}

resource "aws_apigatewayv2_route" "vf_stats_route" {
  api_id = aws_apigatewayv2_api.monmetrecarre_api.id
  route_key = "GET /vf/stats/{codeInsee}"

  target = "integrations/${aws_apigatewayv2_integration.vf_stats_integration.id}"

  authorizer_id   = aws_apigatewayv2_authorizer.monmetrecarre_authorizer.id
  authorization_type = "CUSTOM"
}

resource "aws_apigatewayv2_stage" "prod" {
  api_id = aws_apigatewayv2_api.monmetrecarre_api.id
  name   = "prod"
  auto_deploy = true

  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_lambda_permission" "vf_stats_invoke_permission" {
  statement_id  = "AllowAPIGatewayInvokeVFStats"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.vf_stats_lambda.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.monmetrecarre_api.execution_arn}/*/*"
}

resource "aws_lambda_permission" "authorizer_invoke_permission" {
  statement_id  = "AllowAPIGatewayInvokeAuthorizer"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.lambda_authorizer.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_apigatewayv2_api.monmetrecarre_api.execution_arn}/*"
}

output "base_url" {
  value = aws_apigatewayv2_stage.prod.invoke_url
}
