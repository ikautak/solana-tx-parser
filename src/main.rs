use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{
    EncodedTransaction, UiInnerInstructions, UiMessage, UiTransactionEncoding,
    UiTransactionTokenBalance,
};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(version, about = "solana tx parser")]
struct Args {
    #[clap(short, long)]
    tx_signature: String,
}

fn main() {
    let args = Args::parse();

    //let url = "https://api.devnet.solana.com";
    let url = "https://api.mainnet-beta.solana.com";
    let rpc_client = RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());

    let tx_sig = Signature::from_str(&args.tx_signature).unwrap();

    let config: RpcTransactionConfig = {
        RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: None,
            max_supported_transaction_version: Some(0),
        }
    };
    let tx = rpc_client
        .get_transaction_with_config(&tx_sig, config)
        .unwrap();

    if let Some(v) = tx.transaction.version {
        println!("version {:?}", v);
    } else {
        println!("version undefined");
    }

    let meta = tx.transaction.meta.unwrap();

    // inner instructions
    {
        let inner_instructions: Option<Vec<UiInnerInstructions>> = meta.inner_instructions.into();
        let inner_instructions = inner_instructions.unwrap();
        println!("inner instructions {}", inner_instructions.len());
    }

    let message = match tx.transaction.transaction {
        EncodedTransaction::Json(v) => match v.message {
            UiMessage::Parsed(v) => v,
            _ => {
                println!("no message");
                return;
            }
        },
        _ => {
            println!("no message");
            return;
        }
    };
    let account_keys = message.account_keys;
    //println!("account_keys {}", account_keys.len());

    // SOL balance
    if meta.post_balances.len() > 0 {
        for i in 0..meta.post_balances.len() {
            let pre = meta.pre_balances[i];
            let post = meta.post_balances[i];
            if pre < post {
                println!("index {} sol +{}", i, post - pre);
                println!("{}", account_keys[i].pubkey);
            }
        }
    }

    // Token balance
    let pre_token_balances: Option<Vec<UiTransactionTokenBalance>> = meta.pre_token_balances.into();
    let pre_token_balances = pre_token_balances.unwrap();
    let post_token_balances: Option<Vec<UiTransactionTokenBalance>> =
        meta.post_token_balances.into();
    let post_token_balances = post_token_balances.unwrap();

    if post_token_balances.len() > 0 {
        for i in 0..post_token_balances.len() {
            //
            let mut pre: u64 = 0;
            if pre_token_balances.len() > i {
                pre = u64::from_str(pre_token_balances[i].ui_token_amount.amount.as_str()).unwrap();
            }
            let post =
                u64::from_str(post_token_balances[i].ui_token_amount.amount.as_str()).unwrap();

            if pre < post {
                let change_index = post_token_balances[i].account_index as usize;
                println!("index {} token +{}", change_index, post - pre);
                println!("mint {}", post_token_balances[i].mint);
                let owner: Option<String> =
                    <OptionSerializer<String> as Clone>::clone(&post_token_balances[i].owner)
                        .into();
                let owner = owner.unwrap();
                println!("owner {}", owner);
                //println!("{:?}", account_keys[change_index]);
            }
        }
    }
}
