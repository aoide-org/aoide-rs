// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    num::{NonZeroU32, NonZeroU64},
    sync::Arc,
};

use aoide_repo_sqlite::initialize_connection;
use aoide_storage_sqlite::connection::{
    Storage,
    pool::{
        create_connection_pool,
        gatekeeper::{Config as GatekeeperConfig, Gatekeeper},
    },
};

use super::*;

fn collect_openapi() -> utoipa::openapi::OpenApi {
    let connection_pool =
        create_connection_pool(&Storage::InMemory, NonZeroU32::MIN, initialize_connection).unwrap();
    let gatekeeper = Gatekeeper::new(
        connection_pool,
        GatekeeperConfig {
            acquire_read_timeout_millis: NonZeroU64::MIN,
            acquire_write_timeout_millis: NonZeroU64::MIN,
        },
    );
    crate::runtime::collect_openapi(Arc::new(gatekeeper))
}

fn collect_refs<'a>(value: &'a Value, refs: &mut Vec<&'a str>) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                refs.push(reference);
            }
            object.values().for_each(|value| collect_refs(value, refs));
        }
        Value::Array(array) => array.iter().for_each(|value| collect_refs(value, refs)),
        _ => (),
    }
}

#[test]
fn json_schema_components_replace_all_placeholders() {
    let openapi = collect_openapi();
    let json_schema_components = generate_json_schema_components();
    let placeholders = openapi
        .components
        .as_ref()
        .unwrap()
        .schemas
        .keys()
        .filter(|name| !json_schema_components.contains_key(*name))
        .collect::<Vec<_>>();
    assert!(placeholders.is_empty(), "{placeholders:?}");
}

#[test]
fn all_references_are_resolvable() {
    let document = finish_document(collect_openapi());
    let mut refs = Vec::new();
    collect_refs(&document, &mut refs);
    assert_ne!(refs, Vec::<&str>::new());
    for reference in refs {
        let pointer = reference
            .strip_prefix('#')
            .unwrap_or_else(|| panic!("external reference {reference}"));
        assert!(
            document.pointer(pointer).is_some(),
            "unresolvable reference {reference}"
        );
    }
}

#[test]
fn all_path_parameters_are_documented() {
    let document = finish_document(collect_openapi());
    let paths = document["paths"].as_object().unwrap();
    assert!(!paths.is_empty());
    for (path, path_item) in paths {
        let path_params = path
            .split('/')
            .filter_map(|segment| segment.strip_prefix('{')?.strip_suffix('}'))
            .collect::<Vec<_>>();
        for (method, operation) in path_item.as_object().unwrap() {
            let documented_path_params = operation
                .get("parameters")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .filter(|param| param["in"] == "path")
                .map(|param| param["name"].as_str().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(path_params, documented_path_params, "{method} {path}");
        }
    }
}

#[test]
fn query_parameters_are_converted_losslessly() {
    fn assert_lossless<T: JsonSchema>() {
        let schema = inline_json_schema::<T>();
        let parameters = query_parameters::<T>();
        let properties = schema["properties"].as_object().unwrap();
        assert_eq!(properties.len(), parameters.len());
        for (parameter, (name, mut expected_schema)) in
            parameters.into_iter().zip(properties.clone())
        {
            assert_eq!(parameter.name, name);
            if let Some(description) = expected_schema
                .as_object_mut()
                .unwrap()
                .remove("description")
            {
                assert_eq!(parameter.description.as_deref(), description.as_str());
            }
            assert_eq!(
                serde_json::to_value(parameter.schema).unwrap(),
                non_nullable(expected_schema),
                "{name}"
            );
        }
    }
    assert_lossless::<api::collection::load_all::QueryParams>();
    assert_lossless::<api::track::search::QueryParams>();
    assert_lossless::<api::track::replace::QueryParams>();
    assert_lossless::<api::track::import_and_replace::QueryParams>();
    assert_lossless::<api::playlist::load_all::QueryParams>();
    assert_lossless::<api::EntityRevQueryParams>();
    assert_lossless::<storage::CleanseQueryParams>();
}

#[test]
fn optional_query_parameters_are_not_nullable() {
    let parameters = query_parameters::<api::collection::load_all::QueryParams>();
    let summary = parameters
        .iter()
        .find(|parameter| parameter.name == "summary")
        .unwrap();
    assert!(summary.required == Required::False);
    assert_eq!(
        serde_json::to_value(&summary.schema).unwrap(),
        serde_json::json!({ "type": "boolean" })
    );
}

#[test]
fn operation_ids_are_unique() {
    let document = finish_document(collect_openapi());
    let mut operation_ids = document["paths"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|path_item| path_item.as_object().unwrap().values())
        .map(|operation| operation["operationId"].as_str().unwrap())
        .collect::<Vec<_>>();
    let count = operation_ids.len();
    operation_ids.sort_unstable();
    operation_ids.dedup();
    assert_eq!(count, operation_ids.len());
}
