//! `CreateAccount` subcommand

#![allow(clippy::never_loop)]

use abscissa_core::{Command, Options, Runnable};
use libra_types::transaction::{Script, SignedTransaction};
use crate::{entrypoint, sign_tx::sign_tx, submit_tx::{tx_params_wrapper, batch_wrapper, TxParams}};
use dialoguer::Confirm;
use std::{path::PathBuf, process::exit};
use ol_types::{autopay::PayInstruction, config::{TxType, IS_TEST}};

/// command to submit a batch of autopay tx from file
#[derive(Command, Debug, Default, Options)]
pub struct AutopayBatchCmd {
    #[options(short = "f", help = "path of autopay_batch_file.json")]
    autopay_batch_file: PathBuf,
}


impl Runnable for AutopayBatchCmd {
    fn run(&self) {
        // Note: autopay batching needs to have id numbers to each instruction.
        // will not increment automatically, since this can lead to user error.
        let entry_args = entrypoint::get_args();
        let tx_params = tx_params_wrapper(TxType::Cheap).unwrap();

        let epoch = crate::epoch::get_epoch(&tx_params);
        println!("The current epoch is: {}", epoch);
        let instructions = PayInstruction::parse_autopay_instructions(&self.autopay_batch_file, Some(epoch)).unwrap();
        let scripts = process_instructions(instructions);
        batch_wrapper(scripts, &tx_params, entry_args.no_send, entry_args.save_path)
    }
}

/// Process autopay instructions into scripts
pub fn process_instructions(instructions: Vec<PayInstruction>) -> Vec<Script> {
    // TODO: Check instruction IDs are sequential.
    instructions.into_iter().filter_map(|i| {
      // double check transactions
        match i.type_move.unwrap()<= 3 {
            true => {},
            false => {
              println!("Instruction type not valid for transactions: {:?}", &i);
              exit(1); 
            },
        }
        match i.duration_epochs.unwrap() > 0 {
            true => {},
            false => {
              println!("Instructions must have duration greater than 0. Exiting. Instruction: {:?}", &i);
              exit(1);
            },
        }

        println!("{}", i.text_instruction());
        // accept if CI mode.
        if *IS_TEST { return Some(i) }            
        
        // check the user wants to do this.
        match Confirm::new().with_prompt("").interact().unwrap() {
          true => Some(i),
          _ =>  {
            panic!("Autopay configuration aborted. Check batch configuration file or template");
          }
        }            
    })
    .map(|i| {
      transaction_builder::encode_autopay_create_instruction_script(
        i.uid, 
        i.type_move.unwrap(), 
        i.destination, 
        i.end_epoch.unwrap(), 
        i.value_move.unwrap()
      )
    })
    .collect()
}
 
/// return a vec of signed transactions
pub fn sign_instructions(scripts: Vec<Script>, starting_sequence_num: u64, tx_params: &TxParams) -> Vec<SignedTransaction>{
  scripts.into_iter()
  .enumerate()
  .map(|(i, s)| {
    let seq = i as u64 + starting_sequence_num;
    sign_tx(&s, tx_params, seq, tx_params.chain_id).unwrap()
    })
  .collect()
}

#[test]
fn test_instruction_script_match() {
  use libra_types::account_address::AccountAddress;
  use ol_types::autopay::InstructionType;
  let script = transaction_builder::encode_autopay_create_instruction_script(
    1, 
    0, 
    AccountAddress::ZERO, 
    10, 
    1000);

  let instr = PayInstruction {
      uid: 1,
      type_of: InstructionType::PercentOfBalance,
      destination: AccountAddress::ZERO,
      end_epoch: Some(10),
      duration_epochs: None,
      note: Some("test".to_owned()),
      type_move: Some(0),
      value: 10f64,
      value_move: Some(1000u64),
  };

  instr.check_instruction_match_tx(script).unwrap();

}