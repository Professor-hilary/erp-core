use anyhow::{bail, Result};
use cxx::let_cxx_string;
use std::collections::HashMap;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("recommendation_engine/recommendation_engine.h");

        type Recommendation;

        fn create_engine(config_path: &CxxString) -> UniquePtr<Recommendation>;

        fn recommend(
            engine: &Recommendation,
            context: &CxxString,
            // For simplicity: pass numeric as JSON string, parse in C++
            numeric_json: &CxxString,
            categorical_json: &CxxString,
        ) -> UniquePtr<Recommendation>;

        fn get_action(r: &Recommendation) -> &CxxString;
        fn get_confidence(r: &Recommendation) -> f64;
        fn get_rationale(r: &Recommendation) -> &CxxString;
    }
}

#[derive(Debug)]
pub struct RecResult {
    pub action: String,
    pub confidence: f64,
    pub rationale: String,
}

pub fn get_recommendation(
    config_path: &str,
    context: &str,
    numeric: &HashMap<String, f64>,
    categorical: &HashMap<String, String>,
) -> Result<RecResult> {
    let_cxx_string!(cfg = config_path);
    let engine = ffi::create_engine(&cfg);

    // Serialize maps to JSON strings (use serde_json in real code)
    let num_json = serde_json::to_string(numeric)?;
    let cat_json = serde_json::to_string(categorical)?;

    let_cxx_string!(ctx = context);
    let_cxx_string!(njs = &num_json);
    let_cxx_string!(cjs = &cat_json);

    let result = ffi::recommend(&engine, &ctx, &njs, &cjs);

    if result.is_null() {
        bail!("Engine returned null");
    }

    Ok(RecResult {
        action: ffi::get_action(&result).to_string_lossy().into_owned(),
        confidence: ffi::get_confidence(&result),
        rationale: ffi::get_rationale(&result).to_string_lossy().into_owned(),
    })
}

/*
Usage:
use decision_engine_bridge::get_recommendation;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let mut num = HashMap::new();
    num.insert("customer_ltv".to_string(), 0.85);
    num.insert("stock_level".to_string(), 0.92);

    let mut cat = HashMap::new();
    cat.insert("product_category".to_string(), "electronics".to_string());

    let rec = get_recommendation(
        "config.json",
        "product_upsell",
        &num,
        &cat,
    )?;

    println!("Action: {}", rec.action);
    println!("Confidence: {:.1}%", rec.confidence * 100.0);
    println!("Rationale: {}", rec.rationale);

    Ok(())
}
*/
