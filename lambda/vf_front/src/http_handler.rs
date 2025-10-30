use askama::Template;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb as dynamodb;
use lambda_http::{Body, Error, Request, RequestExt, Response};
use minify_html::{minify, Cfg};
use serde::Deserialize;
use std::collections::HashMap;

type DynamoDbItem = HashMap<String, aws_sdk_dynamodb::types::AttributeValue>;

const GEO_GOUV_API_URL: &str = "https://geo.api.gouv.fr/communes/";

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
    let path_params = event.path_parameters();
    let query = match (
        path_params.first("codeInsee"),
        path_params.first("codePostal"),
        path_params.first("ville"),
    ) {
        (Some(code_insee), _, _) => DynamoQuery::ByCodeInsee(code_insee),
        (_, Some(code_postal), _) => DynamoQuery::ByCodePostal(code_postal),
        (_, _, Some(ville)) => DynamoQuery::ByVille(ville),
        _ => return Err("code_insee, code_postal or ville is required in the path".into()),
    };

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_table: &str =
        &std::env::var("DYNAMODB_TABLE_NAME").unwrap_or("monmetrecarre".to_string());

    let reqwest_client = reqwest::Client::new();
    let geo_gouv_response = reqwest_client
        .get(format!("{GEO_GOUV_API_URL}{}", query.value()))
        .send()
        .await?
        .json::<ApiGeoGouvResponse>()
        .await?;

    let dynamo_client = dynamodb::Client::new(&config);

    let dynamo_results = dynamo_client
        .query()
        .table_name(dynamo_table)
        .key_condition_expression("#attr = :v1")
        .expression_attribute_names("#attr", query.attribute_name())
        .expression_attribute_values(
            ":v1",
            query.dynamo_value(),
        )
        .send()
        .await?;

    let items_as_vec: Vec<DynamoDbItem> =
        dynamo_results.items.expect("No value for input").to_vec();
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
        ville: geo_gouv_response.nom,
        appartements: appartements.clone(),
        maisons: maisons.clone(),
        latest_appartement: latest_appartement,
        code_postaux: geo_gouv_response.code_postaux,
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
