use {super::*, base64::Engine, bitcoin::psbt::Psbt};

const MAX_BURNA_MOUNT: f64 = 10000.0_f64;

pub(super) fn sign_transaction(
  wallet: &Wallet,
  unsigned_transaction: Transaction,
  dry_run: bool,
) -> Result<(Txid, String, u64)> {
  let unspent_outputs = wallet.utxos();

  let (txid, psbt) = if dry_run {
    let psbt = wallet
      .bitcoin_client()
      .wallet_process_psbt(
        &base64::engine::general_purpose::STANDARD
          .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
        Some(false),
        None,
        None,
      )?
      .psbt;

    (unsigned_transaction.txid(), psbt)
  } else {
    let psbt = wallet
      .bitcoin_client()
      .wallet_process_psbt(
        &base64::engine::general_purpose::STANDARD
          .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
        Some(true),
        None,
        None,
      )?
      .psbt;

    let signed_tx = wallet
      .bitcoin_client()
      .finalize_psbt(&psbt, None)?
      .hex
      .ok_or_else(|| anyhow!("unable to sign transaction"))?;

    (
      send_raw_transaction_ext(wallet, &signed_tx, None, Some(MAX_BURNA_MOUNT))?,
      psbt,
    )
  };

  let mut fee = 0;
  for txin in unsigned_transaction.input.iter() {
    let Some(txout) = unspent_outputs.get(&txin.previous_output) else {
      panic!("input {} not found in utxos", txin.previous_output);
    };
    fee += txout.value;
  }

  for txout in unsigned_transaction.output.iter() {
    fee = fee.checked_sub(txout.value).unwrap();
  }

  Ok((txid, psbt, fee))
}

pub fn send_raw_transaction_ext<R: bitcoincore_rpc::RawTx>(
  wallet: &Wallet,
  tx: R,
  maxfeerate: Option<f64>,
  maxburnamount: Option<f64>
) -> Result<bitcoin::Txid> {
  let bitcoin_client = wallet.bitcoin_client();

  // Prepare the parameters for the RPC call
  let mut params = vec![tx.raw_hex().into()];

  // Add maxfeerate and maxburnamount to the params if they are Some
  if let Some(feerate) = maxfeerate {
      params.push(serde_json::to_value(feerate).unwrap());
  } else {
      params.push(serde_json::to_value(0.10).unwrap());
  }

  if let Some(burnamount) = maxburnamount {
      params.push(serde_json::to_value(burnamount).unwrap());
  } else {
      params.push(serde_json::to_value(0.0).unwrap());
  }

  // Make the RPC call
  let tx_id: bitcoin::Txid = bitcoin_client.call("sendrawtransaction", &params)?;

  Ok(tx_id)
}