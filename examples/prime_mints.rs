use futures::future;
use rand::{prelude::ThreadRng, Rng};
use sp_core::Pair;
use sugarfunge_api_types::{primitives::*, *};
use sugarfunge_test::request::*;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get an operator seed
    let operator = format!("//Alice");
    let operator_account = sp_core::sr25519::Pair::from_string(operator.as_str(), None).unwrap();
    println!("Operator:");
    println!("   {}", operator_account.public());

    let mut rng = rand::thread_rng();

    // Generate random accounts
    let accounts: Vec<sp_core::sr25519::Pair> = (0..10)
        .map(|_| {
            let seed = format!("//Account{}", rng.gen_range(100000..1000000));
            sp_core::sr25519::Pair::from_string(seed.as_str(), None).unwrap()
        })
        .collect();

    println!("Accounts:");

    let all_mints: Vec<(
        usize,
        sp_core::sr25519::Pair,
        Vec<(ClassId, AssetId, Balance)>,
    )> = accounts
        .into_iter()
        .enumerate()
        .map(|(i, account)| (i, account, gen_mints(&mut rng)))
        .collect();

    let all_mints = all_mints.into_iter().map(|(i, account, mints)| {
        let operator = operator.clone();

        async move {
            println!("{:02}: {}", i, account.public());

            let mints = mints
                .iter()
                .enumerate()
                .map(|(i, (class_id, asset_id, balance))| {
                    let operator = operator.clone();
                    let account = account.clone();

                    async move {
                        sleep(Duration::from_millis((i * 100) as u64)).await;
                        let res: Result<asset::MintOutput, _> = req(
                            "asset/mint",
                            asset::MintInput {
                                seed: Seed::from(operator.clone()),
                                // seed: Seed::from("".to_string()),
                                to: Account::from(account.public().to_string()),
                                // to: Account::from("".to_string()),
                                class_id: class_id.clone(),
                                asset_id: asset_id.clone(),
                                amount: balance.clone(),
                            },
                        )
                        .await;

                        println!("\t{:02}: {:?}", i, res);
                    }
                });

            future::join_all(mints).await;
        }
    });

    // Random mints for account
    for mint in all_mints {
        mint.await
    }

    Ok(())
}

struct AssetBalances {
    class_ids: Vec<ClassId>,
    asset_ids: Vec<Vec<AssetId>>,
    balances: Vec<Vec<Balance>>,
}

fn gen_mints(rng: &mut ThreadRng) -> Vec<(ClassId, AssetId, Balance)> {
    // Assets mints
    let mints = AssetBalances {
        class_ids: [1000, 1001, 1002, 1003]
            .map(|id| ClassId::from(id))
            .to_vec(),
        asset_ids: [
            [110, 120, 130].map(|id| AssetId::from(id)).to_vec(),
            [210, 220, 230].map(|id| AssetId::from(id)).to_vec(),
            [310, 320, 330].map(|id| AssetId::from(id)).to_vec(),
            [410, 420, 430].map(|id| AssetId::from(id)).to_vec(),
        ]
        .to_vec(),
        balances: [
            (0..3)
                .map(|_| Balance::from(rng.gen_range(100000..1000000)))
                .collect(),
            (0..3)
                .map(|_| Balance::from(rng.gen_range(100000..1000000)))
                .collect(),
            (0..3)
                .map(|_| Balance::from(rng.gen_range(100000..1000000)))
                .collect(),
            (0..3)
                .map(|_| Balance::from(rng.gen_range(100000..1000000)))
                .collect(),
        ]
        .to_vec(),
    };

    let mints = (0..4)
        .map(move |i| {
            (
                mints.class_ids[i],
                mints.asset_ids[i].clone(),
                mints.balances[i].clone(),
            )
        })
        .flat_map(|(class_id, asset_ids, balances)| {
            asset_ids
                .into_iter()
                .zip(balances)
                .map(move |(asset_id, balance)| (class_id, asset_id, balance))
        });
    mints.collect()
}
