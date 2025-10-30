use askama::Template;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{self as dynamodb, types::AttributeValue};
use lambda_http::{Body, Error, Request, Response};
use minify_html::{minify, Cfg};
use std::collections::HashMap;

type DynamoDbItem = HashMap<String, aws_sdk_dynamodb::types::AttributeValue>;

enum DynamoQuery<'a> {
    ByCodeInsee(&'a str),
    ByCodePostal(&'a str),
    ByVille(&'a str),
}

impl DynamoQuery<'_> {
    fn attribute_name(&self) -> &str {
        match self {
            DynamoQuery::ByCodeInsee(_) => "code_insee",
            DynamoQuery::ByCodePostal(_) => "code_postal",
            DynamoQuery::ByVille(_) => "ville",
        }
    }

    fn value(&self) -> &str {
        match self {
            DynamoQuery::ByCodeInsee(v) => v,
            DynamoQuery::ByCodePostal(v) => v,
            DynamoQuery::ByVille(v) => v,
        }
    }

    fn dynamo_value(&self) -> dynamodb::types::AttributeValue {
        dynamodb::types::AttributeValue::S(self.value().to_string())
    }
}

#[derive(Template)]
#[template(path = "vf.html")]
struct VfTemplate<'a> {
    appartements: Vec<&'a DynamoDbItem>,
    maisons: Vec<&'a DynamoDbItem>,
    latest_appartement: Option<&'a DynamoDbItem>,
    latest_maison: Option<&'a DynamoDbItem>,
    code_postaux: Vec<String>,
    ville: String,
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let path = event.uri().path();
    if path.is_empty() || path == "/" {
        return Err("no path param found".into());
    }

    let real_path = &path[1..];
    let Some((param, value)) = real_path.split_once('/') else {
        return Err("invalid path param format".into());
    };

    let query = match param {
        "codeInsee" => DynamoQuery::ByCodeInsee(value),
        "codePostal" => DynamoQuery::ByCodePostal(value),
        "ville" => DynamoQuery::ByVille(value),
        _ => return Err("invalid path param".into()),
    };

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let cities_table: &str =
        &std::env::var("CITIES_DYNAMODB_TABLE_NAME").unwrap_or("cities".to_string());
    let dynamo_vf_table: &str =
        &std::env::var("VF_DYNAMODB_TABLE_NAME").unwrap_or("monmetrecarre".to_string());
    let dynamo_client = dynamodb::Client::new(&config);

    let geo_gouv_result = dynamo_client
        .get_item()
        .table_name(cities_table)
        .key("code", AttributeValue::S(value.to_string()))
        .send()
        .await?;

    let Some(geo_gouv_item) = geo_gouv_result.item else {
        return Err("No item found for input".into());
    };

    let Some(code_insee) = geo_gouv_item.get("code") else {
        return Err("No code_insee found in item".into());
    };

    let vf_results = dynamo_client
        .query()
        .table_name(dynamo_vf_table)
        .key_condition_expression("#code_insee = :v1")
        .expression_attribute_names("#code_insee", "code_insee")
        .expression_attribute_values(":v1", code_insee.clone())
        .send()
        .await?;

    let ville = geo_gouv_item
        .get("nom")
        .and_then(|v| v.as_s().ok())
        .ok_or("Unknown City".to_string())?
        .to_string();
    let code_postaux = geo_gouv_item
        .get("codesPostaux")
        .and_then(|v| v.as_l().ok())
        .ok_or("No code_postaux found".to_string())?
        .iter()
        .filter_map(|av| av.as_s().ok())
        .cloned()
        .collect::<Vec<String>>();

    let items_as_vec: Vec<DynamoDbItem> = vf_results.items.expect("No value for input").to_vec();
    if items_as_vec.is_empty() {
        panic!("No items found for code_insee");
    }
    let (appartements, maisons): (Vec<&DynamoDbItem>, Vec<&DynamoDbItem>) =
        items_as_vec.iter().partition(|e| {
            e.get("type_local")
                == Some(&dynamodb::types::AttributeValue::S(
                    "Appartement".to_string(),
                ))
        });
    let latest_appartement = appartements.last().cloned();
    let latest_maison = maisons.last().cloned();

    let response = VfTemplate {
        ville,
        appartements: appartements.clone(),
        maisons: maisons.clone(),
        latest_appartement: latest_appartement,
        code_postaux,
        latest_maison: latest_maison,
    };

    let rendered = response.render()?;
    let cfg = Cfg {
        minify_js: true,
        minify_css: true,
        ..Cfg::default()
    };
    let minified_bytes = minify(rendered.as_bytes(), &cfg);

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(minified_bytes.into())
        .map_err(Box::new)?;
    Ok(resp)
}
