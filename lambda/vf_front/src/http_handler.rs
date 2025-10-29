use askama::Template;
use aws_sdk_dynamodb as dynamodb;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use minify_html::{minify, Cfg};
use serde::Deserialize;
use std::collections::HashMap;

type DynamoDbItem = HashMap<String, aws_sdk_dynamodb::types::AttributeValue>;

const GEO_GOUV_API_URL: &str = "https://geo.api.gouv.fr/communes/";

#[derive(Template)]
#[template(path = "vf.html")]
struct VfTemplate<'a> {
    appartements: Vec<&'a DynamoDbItem>,
    maisons: Vec<&'a DynamoDbItem>,
    latest_appartement: DynamoDbItem,
    latest_maison: DynamoDbItem,
    code_postaux: Vec<String>,
    ville: String,
}

#[derive(Deserialize)]
struct ApiGeoGouvResponse {
    nom: String,
    #[serde(rename(deserialize = "codesPostaux"))]
    code_postaux: Vec<String>,
    #[serde(rename(deserialize = "population"))]
    _population: u32,

    #[serde(flatten)]
    _extra: HashMap<String, serde_json::Value>,
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let path = event.uri().path();
    if path.is_empty() || path == "/" {
        return Err("code_insee is required in the path".into());
    }
    let code_insee = &path[1..];

    if code_insee.len() != 5 {
        return Err("code_insee must be 5 characters long".into());
    } else if !code_insee.chars().all(|c| c.is_ascii_digit()) {
        return Err("code_insee must be numeric".into());
    }

    let config = aws_config::load_from_env().await;
    let dynamo_table: &str =
        &std::env::var("DYNAMODB_TABLE_NAME").unwrap_or("monmetrecarre".to_string());

    let reqwest_client =reqwest::Client::new();
    let geo_gouv_response =reqwest_client
            .get(format!("{GEO_GOUV_API_URL}{code_insee}"))
            .send()
            .await?
            .json::<ApiGeoGouvResponse>().await?;
        
    let dynamo_client = dynamodb::Client::new(&config);
        
    let dynamo_results = dynamo_client
            .query()
            .table_name(dynamo_table)
            .key_condition_expression("#code_insee = :code_insee")
            .expression_attribute_names("#code_insee", "code_insee")
            .expression_attribute_values(
                ":code_insee",
                dynamodb::types::AttributeValue::S(code_insee.to_string()),
            )
            .send().await?;

    let items_as_vec: Vec<HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> =
        dynamo_results?.items.expect("No value for input").to_vec();
    if items_as_vec.is_empty() {
        panic!("No items found for code_insee {code_insee}");
    }
    let appartements: Vec<&HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> = items_as_vec
        .iter()
        .filter(|e| {
            e.get("type_local")
                == Some(&dynamodb::types::AttributeValue::S(
                    "Appartement".to_string(),
                ))
        })
        .collect::<Vec<_>>();
    let maisons = items_as_vec
        .iter()
        .filter(|e| {
            e.get("type_local") == Some(&dynamodb::types::AttributeValue::S("Maison".to_string()))
        })
        .collect::<Vec<_>>();
    let latest_appartement = appartements.last().unwrap();
    let latest_maison = maisons.last().unwrap();

    let response = VfTemplate {
        ville: geo_gouv_response.nom,
        appartements: appartements.clone(),
        maisons: maisons.clone(),
        latest_appartement: (*latest_appartement).clone(),
        code_postaux: geo_gouv_response.code_postaux,
        latest_maison: (*latest_maison).clone(),
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
