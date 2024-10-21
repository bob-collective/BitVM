use bitcoin::{
    absolute, consensus, Amount, EcdsaSighashType, Network, PublicKey, ScriptBuf, Sequence, TapSighashType, Transaction, TxIn, TxOut, Witness, XOnlyPublicKey
};
use serde::{Deserialize, Serialize};

use crate::bridge::scripts::{generate_pay_to_pubkey_script, generate_pay_to_pubkey_script_address};
use crate::signatures::winternitz::PublicKey as WinternitzPublicKey;

use super::{
    super::{
        connectors::{
            connector::*, connector_1::Connector1, connector_3::Connector3, connector_a::ConnectorA, connector_b::ConnectorB,
        },
        contexts::operator::OperatorContext,
        graphs::base::{DUST_AMOUNT, FEE_AMOUNT},
    },
    base::*,
    pre_signed::*,
};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct KickOff2Transaction {
    #[serde(with = "consensus::serde::With::<consensus::serde::Hex>")]
    tx: Transaction,
    #[serde(with = "consensus::serde::With::<consensus::serde::Hex>")]
    prev_outs: Vec<TxOut>,
    prev_scripts: Vec<ScriptBuf>,
}

impl PreSignedTransaction for KickOff2Transaction {
    fn tx(&self) -> &Transaction { &self.tx }

    fn tx_mut(&mut self) -> &mut Transaction { &mut self.tx }

    fn prev_outs(&self) -> &Vec<TxOut> { &self.prev_outs }

    fn prev_scripts(&self) -> &Vec<ScriptBuf> { &self.prev_scripts }
}

impl KickOff2Transaction {
    pub fn new(context: &OperatorContext, input_0: Input) -> Self {
        let mut this = Self::new_for_validation(
            context.network,
            &context.operator_public_key,
            &context.operator_winternitz_public_key,
            &context.operator_taproot_public_key,
            &context.n_of_n_taproot_public_key,
            input_0,
        );

        this.sign_input_0(context);

        this
    }

    pub fn new_for_validation(
        network: Network,
        operator_public_key: &PublicKey,
        operator_winternitz_public_key: &WinternitzPublicKey,
        operator_taproot_public_key: &XOnlyPublicKey,
        n_of_n_taproot_public_key: &XOnlyPublicKey,
        input_0: Input,
    ) -> Self {
        let connector_a = ConnectorA::new(
            network,
            operator_taproot_public_key,
            n_of_n_taproot_public_key,
        );
        let connector_3 = Connector3::new(network, operator_public_key);
        let connector_b = ConnectorB::new(network, n_of_n_taproot_public_key, operator_winternitz_public_key);

        let _input_0 = TxIn {
            previous_output: input_0.outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        };

        let total_output_amount = input_0.amount - Amount::from_sat(FEE_AMOUNT);

        let _output_0 = TxOut {
            value: Amount::from_sat(DUST_AMOUNT),
            script_pubkey: connector_3.generate_address().script_pubkey(),
        };

        let _output_1 = TxOut {
            value: total_output_amount - Amount::from_sat(2 * DUST_AMOUNT),
            script_pubkey: connector_b.generate_taproot_address().script_pubkey(),
        };

        let _output_2 = TxOut {
            value: Amount::from_sat(DUST_AMOUNT),
            script_pubkey: connector_a.generate_taproot_address().script_pubkey(),
        };

        KickOff2Transaction {
            tx: Transaction {
                version: bitcoin::transaction::Version(2),
                lock_time: absolute::LockTime::ZERO,
                input: vec![_input_0],
                output: vec![_output_0, _output_1, _output_2],
            },
            prev_outs: vec![TxOut {
                value: input_0.amount,
                script_pubkey: generate_pay_to_pubkey_script_address(network, operator_public_key)
                    .script_pubkey()
            }],
            prev_scripts: vec![generate_pay_to_pubkey_script(operator_public_key)],
        }
    }

    fn sign_input_0(&mut self, context: &OperatorContext) {
        let input_index = 0;
        pre_sign_p2wsh_input(
            self,
            context,
            input_index,
            EcdsaSighashType::All,
            &vec![&context.operator_keypair],
        );
    }
}

impl BaseTransaction for KickOff2Transaction {
    fn finalize(&self) -> Transaction { self.tx.clone() }
}
