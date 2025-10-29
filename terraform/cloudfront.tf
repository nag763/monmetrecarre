resource "aws_cloudfront_cache_policy" "na_eu_cache_policy" {
  name        = "monmetrecarre-na-eu-cache-policy"
  comment     = "Cache policy for NA/EU regions with 5-day retention"
  min_ttl     = 36000  # 1 day in seconds
  default_ttl = 432000 # 5 days in seconds
  max_ttl     = 432000 # 5 days in seconds

  parameters_in_cache_key_and_forwarded_to_origin {

    headers_config {
      header_behavior = "whitelist"
      headers {
        items = ["Origin", "Authorization"]
      }
    }

    query_strings_config {
      query_string_behavior = "all"
    }

    cookies_config {
      cookie_behavior = "all"
    }
  }
}

resource "aws_cloudfront_origin_request_policy" "api_gateway_origin_request_policy" {
  name    = "monmetrecarre-api-gateway-origin-request-policy"
  comment = "Origin request policy for API Gateway"

  cookies_config {
    cookie_behavior = "all"
  }

  headers_config {
    header_behavior = "none"
  }

  query_strings_config {
    query_string_behavior = "all"
  }
}

resource "aws_cloudfront_distribution" "monmetrecarre_cloudfront" {
  price_class = "PriceClass_100"
  origin {
    domain_name = replace(replace(aws_lambda_function_url.vf_stats_lambda_url.function_url, "https://", ""),"/", "")
    origin_id   = "monmetrecarre-api-gateway"

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  enabled             = true
  is_ipv6_enabled     = true
  comment             = "CloudFront distribution for monmetrecarre API Gateway"
  default_root_object = "index.html"

  default_cache_behavior {
    allowed_methods          = ["GET", "HEAD", "OPTIONS"]
    cached_methods           = ["GET", "HEAD", "OPTIONS"]
    target_origin_id         = "monmetrecarre-api-gateway"
    viewer_protocol_policy   = "redirect-to-https"
    compress                 = true
    cache_policy_id          = aws_cloudfront_cache_policy.na_eu_cache_policy.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.api_gateway_origin_request_policy.id
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
  }

  tags = {
    Name = "monmetrecarre"
  }
}

output "cloudfront_domain_name" {
  value = aws_cloudfront_distribution.monmetrecarre_cloudfront.domain_name
}
