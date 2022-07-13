use futures::future;
use itertools::Itertools;
use serde_json::json;
use sugarfunge_api_types::{primitives::*, *};
use sugarfunge_test::request::*;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get an operator seed
    let seeded: account::SeededAccountOutput = req(
        "account/seeded",
        account::SeededAccountInput {
            seed: Seed::from("//Alice".to_string()),
        },
    )
    .await
    .unwrap();
    println!("{:?}", seeded.seed);
    println!("{:?}", seeded.account);

    let class_ids_range = 1000..1100;
    let asset_ids_range = 0..100;

    // Generate 100 classes
    let class_ids = class_ids_range.clone().enumerate().map(|(i, class_id)| {
        let delay = (i * 100) as u64;

        let seed = seeded.seed.clone();
        let account = seeded.account.clone();
        let class_id = ClassId::from(class_id);

        async move {
            // Check if class exists
            let class_info: Result<asset::ClassInfoOutput, _> =
                req("asset/class_info", asset::ClassInfoInput { class_id }).await;

            // Create class id
            let class_info = if let Ok(class_info) = class_info {
                class_info.info.is_some()
            } else {
                true
            };
            if !class_info {
                sleep(Duration::from_millis(delay)).await;

                println!("creating: {:?}", class_id);
                let create_class = req::<_, asset::CreateClassOutput>(
                    "asset/create_class",
                    asset::CreateClassInput {
                        seed: seed.clone(),
                        owner: account.clone(),
                        class_id: class_id.clone(),
                        metadata: json!({"test":{
                            "id": class_id.clone(),
                            "desc": "A test asset class"
                        }}),
                    },
                )
                .await;
                println!("created: {:#?}", create_class);
                create_class
            } else {
                Err(RequestError {
                    message: json!(format!("Class exists")),
                    description: "".into(),
                })
            }
        }
    });

    let class_ids_time = std::time::Instant::now();
    let class_ids = future::join_all(class_ids).await;
    for class_id in class_ids {
        if let Ok(class_id) = class_id {
            println!("{:#?}", class_id);
        }
    }
    println!(
        "class_ids elapsed: {}ms",
        class_ids_time.elapsed().as_millis()
    );

    let all_assets = class_ids_range
        .flat_map(|class_id| {
            asset_ids_range
                .clone()
                .map(move |asset_id| (class_id, asset_id))
        })
        .chunks(200);

    // Generate 100 assets per class
    let all_assets = all_assets.into_iter().map(|assets| {
        assets.enumerate().map(|(i, (class_id, asset_id))| {
            let seed = seeded.seed.clone();
            // let account = seeded.account.clone();
            let class_id = ClassId::from(class_id);
            let asset_id = AssetId::from(asset_id);

            async move {
                // Check if asset exists
                let asset_info: Result<asset::AssetInfoOutput, _> =
                    req("asset/info", asset::AssetInfoInput { class_id, asset_id }).await;

                // Create asset id
                let asset_info = if let Ok(asset_info) = asset_info {
                    asset_info.info.is_some()
                } else {
                    true
                };
                if !asset_info {
                    let delay = (i * 100) as u64;
                    sleep(Duration::from_millis(delay)).await;

                    println!("creating: {:?} {:?}", class_id, asset_id);
                    let create_asset = req::<_, asset::CreateOutput>(
                        "asset/create",
                        asset::CreateInput {
                            seed: seed.clone(),
                            class_id: class_id.clone(),
                            asset_id: asset_id.clone(),
                            metadata: json!({"test":{
                                "id": asset_id.clone(),
                                "desc": "A test asset"
                            }}),
                        },
                    )
                    .await;
                    println!("created: {:#?}", create_asset);
                    create_asset
                } else {
                    Err(RequestError {
                        message: json!(format!("Class exists")),
                        description: "".into(),
                    })
                }
            }
        })
    });

    let asset_ids_time = std::time::Instant::now();
    for assets in all_assets {
        let all_assets = future::join_all(assets).await;
        for asset in all_assets {
            if let Ok(asset) = asset {
                println!("{:#?}", asset);
            }
        }
    }
    println!(
        "asset_ids elapsed: {}ms",
        asset_ids_time.elapsed().as_millis()
    );

    Ok(())
}
