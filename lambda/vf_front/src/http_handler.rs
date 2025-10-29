use lambda_http::{Body, Error, Request, RequestExt, Response};
use std::collections::HashMap;
use askama::Template;
use aws_sdk_dynamodb as dynamodb;

type DynamoDbItem = HashMap<String, aws_sdk_dynamodb::types::AttributeValue>;

#[derive(Template)]
#[template(path = "vf.html")]
struct VfTemplate <'a> {
    appartements: Vec<&'a DynamoDbItem>,
    maisons: Vec<&'a DynamoDbItem>,
    latest_appartement: DynamoDbItem,
    latest_maison: DynamoDbItem,
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    let path = event.uri().path();
    if path.is_empty() || path == "/" {
        return Err("code_insee is required in the path".into());
    }
    let code_insee = &path[1..]; 

    let dynamo_table: &str =
        &std::env::var("DYNAMODB_TABLE_NAME").unwrap_or("monmetrecarre".to_string());
    let config = aws_config::load_from_env().await;
    let client = dynamodb::Client::new(&config);
    let results = client
        .query()
        .table_name(dynamo_table)
        .key_condition_expression("#code_insee = :code_insee")
        .expression_attribute_names("#code_insee", "code_insee")
        .expression_attribute_values(
            ":code_insee",
            dynamodb::types::AttributeValue::S(code_insee.to_string()),
        )
        .send()
        .await
        .unwrap();

    let items_as_vec: Vec<HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> =
        results.items.expect("No value for input").to_vec();
    if items_as_vec.is_empty() {
        panic!("No items found for code_insee {code_insee}");
    }
    let appartements: Vec<&HashMap<String, aws_sdk_dynamodb::types::AttributeValue>> = items_as_vec.iter().filter(|e| e.get("type_local") == Some(&dynamodb::types::AttributeValue::S("Appartement".to_string()))).collect::<Vec<_>>();
    let maisons = items_as_vec.iter().filter(|e| e.get("type_local") == Some(&dynamodb::types::AttributeValue::S("Maison".to_string()))).collect::<Vec<_>>();
    let latest_appartement = appartements.last().unwrap();
    let latest_maison = maisons.last().unwrap();
    let hello = VfTemplate {
        appartements: appartements.clone(),
        maisons: maisons.clone(),
        latest_appartement: (*latest_appartement).clone(),
        latest_maison: (*latest_maison).clone(),
        
    };

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(hello.render().unwrap().into())
        .map_err(Box::new)?;
    Ok(resp)
}