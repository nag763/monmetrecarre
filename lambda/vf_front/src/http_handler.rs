use askama::Template;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{self as dynamodb, types::AttributeValue};
use lambda_http::{aws_lambda_events::query_map::serde, Body, Error, Request, Response};
use std::collections::HashMap;

type DynamoDbItem = HashMap<String, aws_sdk_dynamodb::types::AttributeValue>;
type PlotData = (Vec<String>, Vec<String>);

// This function contains your duplicated logic
fn extract_plot_data(items: &[&DynamoDbItem]) -> PlotData {
    items
        .iter()
        .map(|e| {
            (
                unmarshall_dynamo::<String>(e.get("month")).unwrap(),
                unmarshall_dynamo::<f32>(e.get("prix_metre_carre_avg"))
                    .unwrap()
                    .to_string(),
            )
        })
        .unzip()
}

fn unmarshall_dynamo<T: std::str::FromStr>(av: Option<&AttributeValue>) -> Option<T> {
    let val = av?;
    match val {
        AttributeValue::S(s) => s.parse::<T>().ok(),
        AttributeValue::N(n) => n.parse::<T>().ok(),
        _ => None,
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
    appartements_plot_data: (Vec<String>, Vec<String>),
    maisons_plot_data: (Vec<String>, Vec<String>),
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

    let ville: String = unmarshall_dynamo(geo_gouv_item.get("nom")).unwrap();
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
    
    let appartements_plot_data = extract_plot_data(&appartements);
    let maisons_plot_data = extract_plot_data(&maisons);

    let response = VfTemplate {
        ville,
        appartements: appartements.clone(),
        maisons: maisons.clone(),
        latest_appartement,
        code_postaux,
        latest_maison,
        appartements_plot_data,
        maisons_plot_data,
    };

    let rendered = response.render()?;

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(rendered.into())
        .map_err(Box::new)?;
    Ok(resp)
}
