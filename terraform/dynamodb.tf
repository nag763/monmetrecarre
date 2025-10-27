resource "aws_dynamodb_table" "monmetrecarre_dynamodb_table" {
  name         = "monmetrecarre"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "code_insee"
  range_key    = "type_year"

  attribute {
    name = "code_insee"
    type = "S"
  }

  attribute {
    name = "type_year"
    type = "S"
  }

  tags = {
    Name = "monmetrecarre"
  }
}
