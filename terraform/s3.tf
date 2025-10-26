resource "aws_s3_bucket" "monmetrecarre_bucket" {
  bucket = "monmetrecarre"

  tags = {
    Name = "monmetrecarre"
  }

  force_destroy = true
}
resource "aws_s3_object" "scripts" {
  bucket = aws_s3_bucket.monmetrecarre_bucket.id
  key    = "scripts/"

  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_s3_object" "scripts_ingest" {
  bucket = aws_s3_bucket.monmetrecarre_bucket.id
  key    = "scripts/ingest.py"
  source = "../etl/ingest.py"
  tags = {
    Name = "monmetrecarre"
  }
}


resource "aws_s3_object" "scripts_transform" {
  bucket = aws_s3_bucket.monmetrecarre_bucket.id
  key    = "scripts/transform.py"
  source = "../etl/transform.py"
  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_s3_object" "transformed" {
  bucket = aws_s3_bucket.monmetrecarre_bucket.id
  key    = "transformed/"


  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_s3_object" "raw" {
  bucket = aws_s3_bucket.monmetrecarre_bucket.id
  key    = "raw/"

  tags = {
    Name = "monmetrecarre"
  }
}

resource "aws_s3_object" "temporary" {
  bucket = aws_s3_bucket.monmetrecarre_bucket.id
  key    = "temporary/"

  tags = {
    Name = "monmetrecarre"
  }
}