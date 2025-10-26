resource "aws_dynamodb_table" "monmetrecarre_dynamodb_table" {
  name         = "monmetrecarre"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "Commune"
  range_key    = "Type#Year"

  attribute {
    name = "Commune"
    type = "S"
  }

  attribute {
    name = "Type#Year"
    type = "S"
  }

  tags = {
    Name = "monmetrecarre"
  }
}
